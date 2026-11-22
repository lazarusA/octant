use crate::i18n::keys::NoticesKey;

pub fn text(key: NoticesKey) -> Option<&'static str> {
    Some(match key {
        NoticesKey::DismissHover => "閉じる",
        NoticesKey::CopyDetails => "詳細をコピー",
        NoticesKey::RevealInFinder => "Finder で表示",
        NoticesKey::OpenFolder => "フォルダーを開く",
        NoticesKey::NoVariablesFound => "変数が見つかりません",
        NoticesKey::CouldntOpenDataset => "データセットを開けません",
        NoticesKey::UnsupportedFile => "非対応のファイル",
        NoticesKey::CouldntLoadData => "データを読み込めません",
        NoticesKey::CoordinatesUnavailable => "座標を利用できません",
        NoticesKey::CoordinatesFailedDetail => {
            "'{}' の座標を読み込めませんでした ({})。軸にはインデックスを表示します"
        }
        NoticesKey::CoastlinesUnavailable => "海岸線を利用できません",
        NoticesKey::VolumeTooLargeForGpu => "ボリュームが GPU には大きすぎます",
        NoticesKey::VolumeLodOff => "このボリュームでは表示に応じた詳細はオフです",
        NoticesKey::VolumeLodOffDetail => {
            "表示中のレベルには {} 個のブリックが必要ですが、アトラスは {} 個で、半分を細かいレベル用に空けています。ボリュームは一つのレベルで描画されます。"
        }
        NoticesKey::VolumeLodUnevenDetail => {
            "レベル同士がセルの揃った整数倍になっておらず、ブリックに必要な条件を満たしません。ボリュームは1つのレベル全体で描画されます。"
        }
        NoticesKey::VolumeLodWideIntegersDetail => {
            "整数値（ラベルIDなど）の範囲が広すぎて、ブリックでは値を区別できません。ボリュームは1つのレベル全体で描画されます。"
        }
        NoticesKey::VolumeLodUnsupportedDetail => {
            "このGPUはボリュームのシェーダーから書き込めず、ブリックに必要な条件を満たしません。ボリュームは1つのレベル全体で描画されます。"
        }
        NoticesKey::VolumeLodBrickFailed => "ボリュームの一部を細かいレベルで読み込めませんでした",
        NoticesKey::VolumeBufferTooLargeDetail => {
            "ボリュームデータのバッファサイズ ({} 要素 / {} バイト) が GPU ストレージバッファの上限 ({}) を超えています"
        }
        NoticesKey::VolumeTextureTooLargeDetail => {
            "ボリューム {}x{}x{} が GPU の3Dテクスチャ上限 ({}) を超えています"
        }
        NoticesKey::VolumeUploadSkipped => "ボリュームのアップロードをスキップしました",
        NoticesKey::VolumeUploadMismatchDetail => "{} 個の値に対してテクスチャは {}x{}x{} です",
        NoticesKey::DataTooLargeForGpuBuffer => "データが GPU バッファには大きすぎます",
        NoticesKey::BufferTooLargeDetail => {
            "{}: データサイズ ({} バイト) が GPU バッファ容量 ({} バイト) を超えているため、書き込みをスキップします"
        }
        NoticesKey::BufferRangeTooLargeDetail => {
            "{}: 要素 {} からの範囲 (+{} 要素) が GPU バッファ容量 ({} バイト) を超えているため、書き込みをスキップします"
        }
        NoticesKey::LinePlotTooLargeForGpu => "ラインプロットが GPU には大きすぎます",
        NoticesKey::LinePayloadTooLargeDetail => {
            "ラインに {} バイト必要ですが、この GPU がバインドできるのは最大 {} です。"
        }
        NoticesKey::FontLoadFailed => "フォントを読み込めませんでした",
        NoticesKey::FontLoadFailedDetail => "{} のテキストが四角で表示される場合があります: {}",
        NoticesKey::OmeMetadataNonConforming => "OME-Zarr メタデータが仕様に準拠していません",
        NoticesKey::LabelIdsBeyond32Bits => {
            "32 ビットを超えるラベル ID は小さい ID と色を共有します"
        }
        NoticesKey::OmeMetadataNonConformingDetail => {
            "バージョン {} のメタデータ（{}）が仕様に従っていません。読み取れた内容を表示します。"
        }
        NoticesKey::PlateWellStopped => "ウェル {}：読み込みが中断されました",
        NoticesKey::PlateWellNotJson => "ウェル {}：メタデータが有効な JSON ではありません",
        NoticesKey::PlateWellNotOme => "ウェル {}：OME-Zarr のウェルではありません",
        NoticesKey::PlateWellNoMetadata => "ウェル {}：ウェルのメタデータがありません",
    })
}
