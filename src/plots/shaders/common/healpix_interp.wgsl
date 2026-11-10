// Interpolation routine across HEALPix diamond boundaries

// No cell: a corner neighbour off the face.
const HEALPIX_NO_CELL: u32 = 0xFFFFFFFFu;

// The (up to) four pixels around corner `corner_uv` of pixel `pix`, within
// its face and clamped to `max_idx`; `HEALPIX_NO_CELL` where there is none.
// The mesh shaders average a buffer over them (`common/mesh3d.wgsl`).
fn healpix_corner_cells(
    pix: u32,
    corner_uv: vec2<f32>,
    nside: u32,
    is_nested: bool,
    max_idx: u32,
) -> vec4<u32> {
    let ns = max(nside, 1u);
    let fc = healpix_pixel_to_face_coords(pix, ns, is_nested);
    let nside_sq = ns * ns;
    let cx = fc.x + u32(round(corner_uv.x));
    let cy = fc.y + u32(round(corner_uv.y));

    let offsets_x = array<i32, 4>(-1, 0, -1, 0);
    let offsets_y = array<i32, 4>(-1, -1, 0, 0);
    var cells = vec4<u32>(HEALPIX_NO_CELL);

    for (var k = 0u; k < 4u; k = k + 1u) {
        let px_cand = i32(cx) + offsets_x[k];
        let py_cand = i32(cy) + offsets_y[k];

        if (px_cand >= 0 && px_cand < i32(ns) && py_cand >= 0 && py_cand < i32(ns)) {
            let cand_in_face = morton_interleave2(u32(px_cand), u32(py_cand));
            var cand_pix = fc.face * nside_sq + cand_in_face;
            if (!is_nested) {
                cand_pix = healpix_nest2ring(ns, cand_pix);
            }
            cells[k] = min(cand_pix, max_idx);
        }
    }
    return cells;
}
