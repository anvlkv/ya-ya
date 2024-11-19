use leptos::*;
use thiserror::Error;

#[derive(Debug, Clone, Copy, Error, PartialEq)]
pub enum YaYaError {
    #[error("Что-то пошло не так. Попробуйте ещё раз.")]
    ClientError,
    #[error("Ошибка. Попробуйте позже.")]
    ServerError,
    #[error("Ошибка. Обновите расширение.")]
    IntegrationError,
    #[error("Неизвестная ошибика")]
    UnknownError,
}

impl From<reqwest::Error> for YaYaError {
    fn from(err: reqwest::Error) -> Self {
        match err.status() {
            Some(status) if status.is_client_error() => Self::ClientError,
            Some(status) if status.is_server_error() => Self::ServerError,
            _ => Self::UnknownError,
        }
    }
}

impl From<serde_json::Error> for YaYaError {
    fn from(_: serde_json::Error) -> Self {
        Self::IntegrationError
    }
}

#[component]
pub fn ErrorView(
    #[prop(optional)] outside_errors: Option<Errors>,
    #[prop(optional)] errors: Option<RwSignal<Errors>>,
    #[prop(into)] on_retry: Callback<()>,
) -> impl IntoView {
    let errors = match outside_errors {
        Some(e) => RwSignal::new(e),
        None => match errors {
            Some(e) => e,
            None => panic!("No Errors found and we expected errors!"),
        },
    };

    let errors = create_memo(move |_| {
        errors
            .get()
            .into_iter()
            .filter_map(|(_, v)| v.downcast_ref::<YaYaError>().cloned())
            .collect::<Vec<_>>()
    });

    view! {
        <div class="ya-ya-error-view">
            <svg
                width="100"
                height="100"
                viewBox="-10 -10 110 110"
                fill="none"
                xmlns="http://www.w3.org/2000/svg"
            >
                <line x1=10 x2=90 y1=10 y2=90 stroke-width="15" stroke="#b22222"/>
                <line x1=10 x2=90 y1=90 y2=10 stroke-width="15" stroke="#b22222"/>
            </svg>
            <For
                each=move || { errors.get().into_iter().enumerate() }
                key=|(index, _error)| *index
                children=move |error| {
                    let error_string = error.1.to_string();
                    view! {
                        <p>{error_string}</p>
                    }
                }
            />
            <button
                on:click=move |_| on_retry.call(())
                class="ya-ya-button-cta"
            >
                "↺ Попробовать ещё раз"
            </button>
        </div>
    }
}
