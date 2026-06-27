В этом документе описана архитектура `spyland`.

# `core`: Основы ~~шпионажа~~ отслеживания

`spyland-core` это важный крейт во всём проекте, почти сердце. Данный крейт очень простой: не имеет
зависимостей, нет `Result` или `Option`, предсказуем и тестируемый.

## События

`spyland` нацелен на поддержку нескольких Wayland (и не только) композиторов, поэтому `core`
абстрагирует события композиторов в `enum Event`. Задача бэкэнда — переводить события своего
композитора в обобщённый `Event`.

> ```rust
> pub enum Event {
>     ActiveWindowChanged(Option<String>),
>     WorkspaceChanged(i32),
>     Idle(bool),
>     Tick,
> }
> ```
>
> [Source](https://github.com/NonExistPlayer/spyland/blob/1465e924feab9587321f1753ce2f2a0e1b374126/core/src/lib.rs#L68-L87)

## Сессии и состояния

Сессии — это что вы можете представить себе под сессией. Говоря более конкретно: это
продолжительность пользовательского состояния. Сессия очень простая структура:

> ```rust
> pub struct Session {
>     pub start: u64,
>     pub end: u64,
> 
>     pub state: State,
> }
> ```
>
> [Source](https://github.com/NonExistPlayer/spyland/blob/1465e924feab9587321f1753ce2f2a0e1b374126/core/src/lib.rs#L25-L38)

Пользовательское состояние описывается в `enum State`:

> ```rust
> pub enum State {
>     Active {
>         app_id: String,
>         workspace: Option<i32>,
>     },
>     Idle,
> }
> ```
>
> [Source](https://github.com/NonExistPlayer/spyland/blob/1465e924feab9587321f1753ce2f2a0e1b374126/core/src/lib.rs#L45-L63)

У пользователя состояние либо активное, либо нет. Rust своим синтаксисом, позволяет удобно добавить
специальные поля только для `State::Active`.

## Менеджер сессий

Со `spyland-core` поставляется в отдельном модуле `manager`, менеджер сессий `SessionManager`.
Он отвечает за обработку абстрагированных событий и на их основе составляет сессии. Модуль
`manager` содержит типы относящиеся, как не странно, к менеджеру сессий.

### `trait Clock`: Абстракция времени

Первое что важно понимать в менеджере сессий и в самом тестировании `core`, это именно этот трейт.
Он содержит только один метод `now()`:

> ```rust
> pub trait Clock {
>     fn now(&self) -> u64;
> }
> ```
>
> [Source](https://github.com/NonExistPlayer/spyland/blob/1465e924feab9587321f1753ce2f2a0e1b374126/core/src/manager.rs#L46-L52)

Такой трейт нужен для того, чтобы менеджер мог работать и с реальным временем и с контролируемым
для тестов. Например, реализация с настоящим временем:

> ```rust
> struct SystemClock;
> 
> impl Clock for SystemClock {
>    fn now(&self) -> u64 {
>         SystemTime::now()
>             .duration_since(UNIX_EPOCH)
>             .expect("time went backwards")
>             .as_secs()
>     }
> }
> ```
>
> [Source](https://github.com/NonExistPlayer/spyland/blob/1465e924feab9587321f1753ce2f2a0e1b374126/daemon/src/main.rs#L86-L95)

Подразумивается, что `now()`, возвращает [UNIX Timestamp](https://en.wikipedia.org/wiki/Unix_time),
однако формат может вариароваться от реализации.

### `struct SessionManager`: Менеджер сессий

Эта структура в качестве generic типа принимает, реализацию `trait Clock`. Основной метод менеджера
это `handle_event()`. Как не трудно догадаться, эта функция принимает `Event` и обрабатывает его.
Исходя из событий, менеджер выполняет действия:

- #### `ActiveWindowChanged(Option<String>)`
Создаёт новую сессию из `app_id` и текущего `workspace`.
**Ответы**: `SessionCreated`, `Ignored` (приложение скрыто конфигурацией).
- #### `WorkspaceChanged(i32)`
Синхронизирует текущий workspace с менеджером. Изменяет приватное поле `workspace`.
**Ответы**: `Handled`.
- #### `Idle(bool)`
Если `true`, сохраняет текущею сессию и создают новую с состоянием `State::Idle`.
Если `false`, заканчивает `Idle` сессию и возобновляет старую.
**Ответы**: `SessionIdled(bool)`, `Handled` (создана idle сессия, не сохранена текущая), `Ignored`.
- #### `Tick`
Время (`Clock::now`) изменилось. Во время этого события, менеджер проверяет если с момента
последнего `flush()` (автоматического) прошло больше/равно чем указано в конфигурации.
**Ответы**: `Handled`, `Ignored`, `Flushed` (произошёл автоматический `flush`).

### `struct Configuration`: Конфигурация менеджера

> ```rust
> pub struct Configuration {
>     pub flush_interval: u64,
>     pub hidden_applications: Vec<String>,
>     pub min_session_duration: u64,
> }
> ```
>
> [Source](https://github.com/NonExistPlayer/spyland/blob/1465e924feab9587321f1753ce2f2a0e1b374126/core/src/manager.rs#L105-L120)

Менеджер сессий может конфигурироваться данной структурой. Она по большей части предназначена для
конфигов spyland, но чтобы не загружать `core`, у крейта присутсвует feature `serde`, которая
добавляет (де)сериализацию у некоторых структур, включая эту.

- #### `flush_interval`
Периодичность автоматического `flush()`. Формат времени также зависит от `trait Clock`.
- #### `hidden_applications`
Вектор `app_id` приложений, который менеджер будет игнорировать.
- #### `min_session_duration`
Минимальная продолжительность сессии. Если она меньше, менеджер проигнорирует её. Формат времени
также зависит от `trait Clock`.

Конфигурацией менеджера можно просматривать и управлять с помощью методов `config()` и
`set_config()` соответственно. По умолчанию, менеджер использует функцию `default()` у конфигурации
для её инициализации:

> ```rust
> impl Default for Configuration {
>     fn default() -> Self {
>         Configuration {
>             flush_interval: 15,
>             hidden_applications: Vec::new(),
>             min_session_duration: 5,
>         }
>     }
> }
> ```
>
> [Source](https://github.com/NonExistPlayer/spyland/blob/1465e924feab9587321f1753ce2f2a0e1b374126/core/src/manager.rs#L122-L130)

### `enum Response`: Ответы

Менеджер в методах `handle_event()` и `flush()` возвращает `Response`. Этот ответ это действие,
которое он выполнил. Для подробного описания, смотрите
[документацию `handle_event()`](https://github.com/NonExistPlayer/spyland/blob/1465e924feab9587321f1753ce2f2a0e1b374126/core/src/manager.rs#L154-L170),
и [документацию `Response`](https://github.com/NonExistPlayer/spyland/blob/1465e924feab9587321f1753ce2f2a0e1b374126/core/src/manager.rs#L65-L98).

### `fn flush()`: Сохранение сессий

Менеджер текущею сессию всегда хранит в приватном поле `current`. Если вы захотите получить сессии
через метод `sessions()`, то вам обязательно придётся сохранить текущею для правильной информации.

Для избежания потери данных, в `SessionManager` существуют автоматические `flush()`, которые
вызываются при `Event::Tick`. Однако, тогда бы сессии хранились бы неэффективно, поэтому во
`flush()` есть автоматическое слияние. Если состояние текущей и последний сессии совпадает,
тогда `flush()` обновляет эту последнею сессию, изменяя `Session::end` на текущее время.

Проверка на минимальную длительность сессии (`min_session_duration`) происходит именно в этой
функции.

### Внутрение поля `SessionManager`

`SessionManager` имеет следующую структуру:

> ```rust
> pub struct SessionManager<C: Clock> {
>     current: Option<Session>,
>     workspace: Option<i32>,
>     clock: C,
>     sessions: Vec<Session>,
>     old_session: Option<Session>,
>     last_flush: u64,
>     config: Configuration,
> }
> ```
>
> [Source](https://github.com/NonExistPlayer/spyland/blob/1465e924feab9587321f1753ce2f2a0e1b374126/core/src/manager.rs#L55-L63)

Краткое описание для каждого поле:

- **`current`**: Текущая сессия. Может быть `None`, если сессия не инициализирована.
- **`workspace`**: Текущий workspace. `None` при инициализации. Изменяет через `WorkspaceChanged`.
- **`clock`**: Экземпляр `C` (`trait Clock`).
- **`sessions`**: Сюда `flush()` сохраняет сессии. Возвращается от `sessions()`.
- **`old_session`**: Используется для сохранения сессии при `Event::Idle`.
- **`last_flush`**: Время последнего автоматического `flush()`.
- **`config`**: Конфигурация менеджера в виде структуры.

## Аналитика сессий

В `core` есть ещё один модуль - `analytics`. Единственный тип внутри — `struct SessionAnalytics`.
Структура для инициализации (`new()`) принимает вектор сессий. Она представляет удобную стастистику
для сессий.

## Тесты

Тестирование — важная часть `spyland`. У `core` больше всего тестов (около 20). Они разделены по
разным файлам:

- **`lib.rs`**: тесты базового функционала
- **`analytic.rs`**: тесты модуля `analytics`
- **`config.rs`**: тесты работы конфигурации `SessionManager`
- **`responses.rs`**: тесты корректности ответов от `SessionManager`

Однако существует ещё один модуль, который предоставляет обёртку над менеджером, для удобного
написания тестов:

### `common.rs`

`common.rs` это модуль в `tests/`, предоставляющий удобную для тестов обёртку над
`SessionManager`, под названием `TestDriver`.

Каждый модуль (файл) в интеграционных тестах, это отдельный исполняемый файл. Поэтому, в каждом
модуле нужно прописывать:

```rust
mod common; // Объявляем модуль
use common::TestDriver; // Используем TestDriver
```

Каждый тест, начинается одинаково — с инициализации `TestDriver`:

```rust
#[test]
fn test_name() {
    // Переменная мутируемая, так как
    // SessionManager будет мутироваться.
    let mut d = TestDriver::new();
}
```

Далее, логика теста и ожидаемое поведение.

> Пример теста:
>
> ```rust
> #[test]
> fn session_data_test() {
>     let mut d = TestDriver::new();
> 
>     // Константы (см. Тесты в CONTRIBUTING.md)
>     const WORKSPACE: i32 = 1;
>     const APP_ID: &str = "firefox";
> 
>     // Имитируем сессию
>     d.event(Event::WorkspaceChanged(WORKSPACE));
>     d.event(Event::ActiveWindowChanged(Some(APP_ID.into())));
>     d.flush(); // Сохранение
> 
>     // Сравниваем состояние первой сессии
>     match &d.mgr.sessions()[0].state {
>         State::Active { app_id, workspace } => {
>             // Сравниваем app_id
>             assert_eq!(APP_ID, app_id, "app_id not matching");
>             // Сравниваем workspace
>             assert_eq!(
>                 WORKSPACE,
>                 workspace.expect("workspace is none"),
>                 "workspace not matching"
>             );
>         }
>         // Неожиданное состояние
>         _ => panic!("Incorrect state"),
>     }
> }
> ```
>
> [Source](https://github.com/NonExistPlayer/spyland/blob/1465e924feab9587321f1753ce2f2a0e1b374126/core/tests/lib.rs#L50-L72)
