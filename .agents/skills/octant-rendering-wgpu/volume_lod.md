# Volume LOD: bricked multiresolution volumes

How "Detail by view" draws a multiscale volume finer where the camera looks,
as built. CPU side in `src/app/volume_lod/`, GPU side in
`src/plots/volume/` and `src/plots/shaders/volume/`.

## When it runs

`VolumePlotConfig::lod` (Settings, "Detail by view"), off by default: off,
nothing is read and the volume draws exactly as before. On, the base layer
is bricked (`setup.rs` `volume_lod_context`) when it is a volume of a
multiscale variable at the Auto level, x, y and z resolved, not playing or
animated along x, y or z. Any mode, composites included. It stays dense,
with a notice once per variable and reason (`reset.rs` `warn_lod_off`,
`setup.rs` `Off`), when the roots would fill more than half the atlas,
when the levels do not brick the table (`Bricks::aligned`: whole factors,
brick edges within half a level cell of a table cell's), for integer ids
spanning more than f16 holds apart (`placement.rs` `wide_integers`), or
when the GPU cannot write storage from the fragment stage. The context hash
(`plot_hash`) changes with the plot; a new context drops every brick,
cancels their reads (never one a layer waits on: a root over a whole small
level reads the plot's own block), and starts over (`reset.rs`
`reset_volume_lod`), reusing the renderer when its atlas fits (the next
step), its frames and ray feedback dropped (`forget_frames`). A composite's settings recolor
its bricks only once they rested `SETTLE` seconds (dragging a contrast
slider rebuilds nothing); turning it on or off rebuilds at once, once the
plotted block it blends from is cached (`plotted_block`, found by the
layer's request: `load.block_key` is cleared when a block is accepted); with that block gone and no read pending, the plot rebuilds
without bricks in its new colors, `wait_for_plotted_block`). The
renderer and its atlas are kept when they fit (`composite.rs`
`recolor_volume_lod`). The plotted
(dense) volume keeps drawing until every root brick has arrived (`ready`).

## Bricks and the table

- A brick is `BRICK` = 64 cells a side of one level (`data/multiscale/
  bricks.rs` `BrickKey { level, b }`), stored with a one-cell apron
  (`STORED` = 66). Reads cover the interior only (`Bricks::window`), so a
  read stays inside the store's chunks wherever chunk edges fall on brick
  edges; the apron is filled from resident same-level neighbours
  (`apron.rs`), the brick's edge repeating where none is resident.
- The table has one cell per 64^3 level-0 cells. At every level a table cell
  maps to the brick holding the cell's center (`cell_brick`), so OME pyramids
  offset by half a cell partition exactly; `cells(key)` are the table cells
  a brick covers, `children(key)` the next level's bricks over them.
- The roots are the plotted level's bricks over the window (`select.rs`
  `root_bricks`), pinned in the atlas.

## Residency (`residency.rs`)

Slots: 8x8x6 = 384 on the desktop, doubled on a discrete GPU, 8x8x3 on the
web, doubled again for a byte atlas (`setup.rs` `atlas_slots`), then cut
along z until the placed bricks' blocks fit in half the block cache
(`cached_slots`: aprons and the hover read them there); more roots than half the slots turns the
plot dense with a notice. `Residency<T>` keeps slot by key, the CPU mirror of
the table (slot and level + 1 per cell, the finest resident level wins), and
per slot the last selection that wanted it. Eviction takes the least
recently wanted slot that is neither pinned nor wanted by the latest
selection (ages count selections, not frames); an evicted brick hands its
cells back to the nearest resident ancestor.

## Selection (`want.rs`, `select.rs`)

Once the view has rested `SETTLE` seconds (`mod.rs` `settle_volume_lod`):

1. Roots first, then children of bricks in the frustum whose voxels span
   more than `PIXELS_PER_VOXEL` (1.5) / quality pixels at their nearest
   point, largest first, within the atlas's slots.
2. `Skip::empty`: only bricks that draw nothing at any level are never
   split: all missing (no NaN color), or all at the plotted volume's
   minimum below the shown range (background, padding). A dim coarse brick
   of a mean pyramid may hold bright finer voxels, so it is split. MIP,
   MinIP and the classic modes (5-7, which color values outside the range)
   show everything; every placed brick widens the ranges of the resident
   ancestors of all its corner cells (`ranges.rs`; offset pyramids give a
   brick two parents along an axis), and a brick placed after finer ones
   under it takes their ranges (`absorb_descendants`: reads run in
   parallel), so coarse ranges become true bounds where finer data was
   seen (the GPU's skipping reads them). A child two parents share is
   chosen once.
3. `Skip::hidden`: bricks over no table cell rays reach are never wanted.
   Cells reached come from the GPU's ray feedback once read back for the
   view resting now (below), else from the CPU's guess (`occlusion.rs`, DVR
   and opaque label surfaces: rays every 32 px through the plotted volume at
   half the shader's opacity, colormap opacity and curve included,
   stopping at 0.95). Visibility travels as `CellBits` (one bit per cell).
4. `Skip::refined` (hysteresis): a brick with finer bricks resident stays
   split until its voxels shrink below `KEEP_SPLIT` (0.7) of the threshold.

The wanted set is touched (kept from eviction), the queue is rebuilt from
what is neither resident, running nor waiting to be placed, and reads of
bricks no longer wanted are cancelled (`fetch.rs` `drop_unwanted_reads`).
`finest` (the finest level wanted and resident) sets the ray's step at rest.

## Reads and placement (`fetch.rs`, `encode.rs`)

- A brick is an ordinary block request of its level through
  `block_prefetcher` (the status bar's spinner shows it), at most
  `IN_FLIGHT` = 8 at once. `BlockPrefetcher::cancel` drops a queued request
  before it starts; a running one ends `unrequested` (`PrefetchResult`):
  cached, its failure only logged, never clearing a newer request for the
  same block; a result that arrives after its cancel is unrequested too
  (each request has an id, so it never answers the key's next request).
  A composite's plotted block is touched as each brick arrives
  (`BlockCache::touch`), so the bricks never evict what recolors them.
  Tables past `TABLE_CELLS` (2^26 cells) stay dense. Reads an abort lost
  are asked again (`requeue_lost_bricks`); the first failure of a plot
  shows a toast. A brick its block does not hold is reported the same way
  and never asked again (`Lod::unreadable`).
- Arrivals wait in `arrived` and are placed every `PLACE_EVERY` (0.1 s)
  within `PLACE_BUDGET` (6 ms) of the frame.
- Encoding (`setup.rs` `atlas_encoding`): scalar bricks as f16 of
  `(v - offset) / scale` over the plotted volume's range in an `R16Float`
  atlas, NaN where missing, saturating past f16's range; 8-bit integer data whose plotted values are
  all whole and within the type's 256, none missing (packed data is not), as
  bytes `(v - offset) / 255` in an `R8Unorm` atlas (exact; converted from the
  f16 cube at upload, `encode.rs` `to_unorm8`; ranges decode what the GPU
  reads, `Encoding::decode`); composites as
  RGBA8 in an `Rgba8Unorm` atlas (`composite.rs`), blended on the CPU by
  `slice_multichannel_volume_stretched` / `slice_rgb_volume_stretched` with
  the stretches recorded once over the plotted block, so every level shares
  the plot's colors (a brick's read spans the channel dimension, so its
  channels arrive together). Composite aprons are blended from the
  neighbour's channels over just the apron's box (`apron_color.rs`).
- Each slot's valid range and missing flag go to a storage buffer
  (`write_range`), and per 8^3 sub-block of its brick (and one cell around)
  to another (`subblocks.rs` `sub_ranges`, `write_subs`, scanned in one pass
  with the brick's range and missing flag), for skipping; composites are
  always open. Scalar bricks and aprons read straight from the block with
  column stride offsets (`apron.rs` `region_values`, `draw.rs`
  `block_axes_for`). Filling a neighbour's apron widens its ranges (`ranges.rs`
  `widen_apron`) and its missing flag when the apron brings NaN; finer
  bricks widen their ancestors' ranges and missing flags (`widen_ancestors`,
  `absorb_descendants`). Placement: `place.rs`.

## GPU (`plots/volume/bricks.rs`, `bricked.rs`, `feedback.rs`)

`VolumeRenderer::new_bricked` builds the bricked pipeline without allocating
dense textures (`textures: None`; `volume_validity` and `volume_blocks` live
in `sample_dense.wgsl` only), over:
binding 1 the atlas, 3 the sampler, 5 the `Bricked` uniforms (window, flips,
grid, value encoding, slots, per level factor, offset and brick count), 6 the
table (`Rgba8Uint`), 7 the slots' ranges, 8 the ray feedback bits, 9 the
sub-block ranges (layout in `bricked.rs`, types in `brick_types.rs`). The shader is
the dense one with `sample_bricked.wgsl` and `walk_bricked.wgsl` in place of
`sample_dense.wgsl` and `walk.wgsl` (same function names); `MAX_SAMPLES` is
8192 for bricked pipelines.

- Sampling: a unit-box position becomes a level-0 position, its table cell
  names slot and level, the brick holding the cell's center is the one in
  the slot, sampled trilinearly inside its stored cube.
- Per-brick steps: the ray's lattice is the finest level drawn; in a coarser
  brick `next_open_sample` takes one sample per `brick_stride` steps and DVR
  and X-ray weigh it by `sample_span`; cells drawing nothing are skipped
  whole, and inside an open cell the 8^3 sub-blocks drawing nothing
  (`next_open_sub`, never past the cell). DVR (scalar and composite), MinIP
  (skipping only blocks wholly below cmin, `OPEN_AT_LEAST`, and only
  without a visible low clip color: values above cmax draw in the top
  color) and X-ray step this way. A coarse brick's samples start where the
  ray enters its cell or 8^3 sub-block (or where a skip lands, `on_stride`),
  never jump past where it ends, and the last one stands for the steps left
  to it (and to the ray's end), so no step is lost, counted twice where
  levels meet, or counted for an empty sub-block or past the ray
  (`sample_bricked_steps.wgsl`). Entering a cell (`enter_cell`) finds its
  brick once; the samples of DVR, MinIP, X-ray and indexed RGBA read it
  directly (`blocks_scalar`, `blocks_exact`, `blocks_composite`), gradient
  taps through the table. The opaque surface search starts at the sample
  taken before (`previous_taken`), a stride back in a coarse brick.
- Missing voxels: in a slot flagged as missing some, `filtered_value`
  weighs the eight trilinear taps by validity as the dense volume's
  validity texture does (missing below half), else hardware filtering.
- MIP and label surfaces walk the voxels (`walk_bricked.wgsl`): table cells
  in order and, in each, every voxel of the level resident there; while
  moving the coarser grid of `set_dims`, reading each coarse cell's center
  voxel. The classic modes 5-7 read texels through `texel_value` and take
  coarse bricks on their own step (`next_strided_sample`, no skipping by
  value: indexed palettes color values outside the range); absorption and
  indexed RGBA compound a sample's opacity over its span
  (`corrected_alpha`), additive RGBA sums it.
- Ray feedback: `mark_seen` sets one bit per table cell a ray enters
  (in `enter_cell` and `walk_enter`) until it stops, only in resting
  (banded) frames (`record_seen`), testing the bit before setting it. The
  first band of
  a resting frame clears the bits, the last copies them for readback
  (`Feedback::record`); the app polls (`take_ray_feedback`, non-blocking
  `map_async`), keeps bits tagged with the resting view's camera and selects
  again with them. Every mode reports; the bits are tagged with the frame's
  parameters, so a frame of another mode, color range or opacity is not
  taken.

## Drawing (`draw.rs`, `render.rs`, `paint_layer.rs`)

- `volume_draw` picks the bricked renderer once ready: moving (the view, or
  any widget such as a slider being dragged), rays step at the plotted
  (root) level and frames draw whole so they keep up; at rest, at
  `finest`.
- A resting frame is built in `RESTING_BANDS` (16) bands of rows over as
  many egui frames into a second frame, shown once complete; bricks arriving
  mid-build join the remaining bands instead of restarting it, and the
  build keeps the data version it started with, so the finished frame is
  rebuilt once with them in every band. With nothing shown yet (a new
  renderer, or one reused for the next plot, `forget_frames`) the first
  frame is drawn whole; an export draws whole too. The paint asks for
  repaints until `shows` (same band clamp as the render), never over a
  canvas of no area. A frame finished while a readback runs keeps
  `waiting` true until its copy is made.
- The frame key adds the bricks' write counter.

## Hover

`brick_reading` names the level and value of the finest resident brick
under the hit voxel (read from the block cache); in MIP, the largest finer
value inside the hit voxel, in MinIP the smallest. The hover keeps the
reading until the voxel, mode, plot or bricks change (`ui::temp_cache`).
Composites read their channels from the plotted volume as before.

## Related fix

The 3D box is scaled to its longest side in any unit (`pipeline/aspect.rs`
`unit_box`); metre-scaled OME stacks were 30x too small to zoom into.

## Tests and benchmarks

- CPU: `bricks_tests.rs` (data), `select_tests.rs` (frustum, budget, empty,
  hysteresis), `residency_tests.rs`, `apron_tests.rs`, `occlusion_tests.rs`,
  `tests.rs` (app over the isotropic pyramid: roots, close-up to level 0,
  hover values, cancelling, aborts, byte round trip, label occlusion),
  `ranges_tests.rs`, `apron_color_tests.rs`, `subblocks.rs` tests,
  `volume_stretch_tests.rs`; `feedback_tests.rs` (ray feedback end to end
  with a GPU).
- GPU: `bricked_gpu_tests.rs` (every mode against the dense renderer,
  flips, coarse bricks, missing voxels), `bricked_walk_gpu_tests.rs`,
  `bricked_composite_gpu_tests.rs`, `bricked_byte_gpu_tests.rs`,
  `bricked_skip_gpu_tests.rs` (sub-block skipping changes nothing),
  `bricked_missing_gpu_tests.rs` (scattered missing voxels as dense),
  `banded_gpu_tests.rs` (bands, arrivals mid-build, ray feedback).
- By hand, in release, `--ignored --nocapture --test-threads 1`:
  `volume_lod_beechnut` (settle times, frame times per mode, image
  comparison, ray feedback rounds) and `brick_fetch_timed` (per level of
  Beechnut, Janelia HeLa, Kingsnake and Woodbranch, one brick with and
  without apron, chunks touched, eight at once).

## Not built yet

- Bricks while playing or animating along x, y or z: a level per step from
  the measured bandwidth, skipping regions that barely changed.
- Refinement ranked by visual gain per byte, from shard indexes'
  compressed chunk sizes and stored chunk statistics.
- Per-brick value ranges kept across sessions (today: this session only)
  and a client-side max pyramid of what was seen.
- A resolution map (today the hover names the level drawn), temporal
  accumulation at rest, tricubic placeholders, reads ordered by the
  pointer.
- A page-table hierarchy past the single-level table's ~131k voxels a side,
  and compressed atlases for data wider than 8 bits.

## Sources

The design follows the published large-volume literature. Principal
sources: ray-guided streaming
(Crassin et al., GigaVoxels, I3D 2009) and visualization-driven virtual
memory (Hadwiger et al., IEEE TVCG 2012), surveyed by Beyer et al. (CGF
2015); indirection into one packed texture (Kraus & Ertl, Graphics Hardware
2002; Beyer et al., VG 2008); adaptive sampling of multiresolution volumes
and transfer-function-based LOD (Ljung, VG 2006; Ljung et al., EuroVis
2006); occlusion clipping (Li, Mueller & Kaufman, Vis 2003); hierarchical DDA
(Museth, 2014); OME-Zarr (Moore et al., 2021, 2023). The closest recent
system is the Residency Octree (Herzberger et al., IEEE TVCG 2024), a web
renderer decoupling cache residency from resolution.
