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
        NoticesKey::FontLoadFailedDetail => "{} のテキストが四角で表示される場合があります：{}",
    })
}
