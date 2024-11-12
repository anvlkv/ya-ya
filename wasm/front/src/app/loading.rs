use leptos::*;

#[component]
pub fn Loading() -> impl IntoView {
    view! {
        <figure class="ya-ya-loading">
            <svg
                width="100"
                height="100"
                viewBox="0 0 100 100"
                fill="none"
                xmlns="http://www.w3.org/2000/svg"
            >
                <path
                    fill-rule="evenodd"
                    clip-rule="evenodd"
                    d="M46.1477 98.8431C47.3675 99.2112 48.5769 99.5898 49.7654 100C50.9538 99.5898 52.1631 99.2112 53.3832 98.8431C67.3803 94.6996 82.8813 93.8688 90.5998 93.7323C93.6915 93.6692 95.5319 93.7323 95.5319 93.7323V50.5095C93.3445 50.4674 91.2201 50.4674 89.1694 50.5095C88.8855 50.5095 88.591 50.52 88.3071 50.52C64.3193 51.1299 49.7659 56.8929 49.7659 56.8929C49.7659 56.8929 35.2115 51.13 11.2247 50.52C10.9408 50.52 10.6464 50.5095 10.3624 50.5095C8.31173 50.4674 6.18739 50.4674 4 50.5095V93.7323C4 93.7323 5.84034 93.6692 8.93206 93.7323C16.6511 93.869 32.151 94.6998 46.1477 98.8431ZM53.3829 91.3344C62.6477 88.8105 72.1861 87.6012 79.7156 87.0226C82.965 86.7702 85.8782 86.6335 88.3074 86.5599V57.7563C78.0435 58.0192 69.6409 59.2812 63.5415 60.5642C59.8188 61.353 56.9584 62.1417 55.0655 62.7201C54.3714 62.9409 53.8035 63.1197 53.3829 63.2669V91.3344ZM46.1477 63.2658V91.3344C36.8828 88.8105 27.3444 87.6012 19.815 87.0226C16.5655 86.7702 13.6523 86.6335 11.2232 86.5599V57.7563C21.4871 58.0192 29.8896 59.2812 35.989 60.5642C39.7117 61.353 42.5722 62.1417 44.465 62.7201C45.1591 62.9409 45.727 63.1186 46.1477 63.2658Z"
                    fill="currentColor"
                />
                <path
                    d="M71.3043 21.5374C71.3043 33.4322 61.6617 43.0748 49.7669 43.0748C37.8721 43.0748 28.2295 33.4322 28.2295 21.5374C28.2295 9.64262 37.8721 0 49.7669 0C61.6617 0 71.3043 9.64262 71.3043 21.5374ZM35.6656 21.5374C35.6656 29.3253 41.979 35.6387 49.7669 35.6387C57.5548 35.6387 63.8682 29.3253 63.8682 21.5374C63.8682 13.7495 57.5548 7.43608 49.7669 7.43608C41.979 7.43608 35.6656 13.7495 35.6656 21.5374Z"
                    fill="currentColor"
                />
            </svg>
            <figcaption>
                Перевожу на Ясный Язык...
            </figcaption>
        </figure>
    }
}
