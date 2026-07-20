В этом документе описана архитектура `spyland`.

# `core`: Основы ~~шпионажа~~ отслеживания

`spyland-core` это ключевой крейт в проекте, как ядро. Данный крейт очень простой: не имеет
зависимостей, нет `Result`, предсказуем и легко тестируем.

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
> [Source](https://github.com/spyland-dev/spyland/blob/1465e924feab9587321f1753ce2f2a0e1b374126/core/src/lib.rs#L68-L87)

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
> [Source](https://github.com/spyland-dev/spyland/blob/1465e924feab9587321f1753ce2f2a0e1b374126/core/src/lib.rs#L25-L38)

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
> [Source](https://github.com/spyland-dev/spyland/blob/1465e924feab9587321f1753ce2f2a0e1b374126/core/src/lib.rs#L45-L63)

У пользователя состояние либо активное, либо нет. Rust своим синтаксисом позволяет удобно добавить
специальные поля только для `State::Active`.

## Менеджер сессий

Вместе со `spyland-core` в отдельном модуле `manager` поставляется менеджер
сессий `SessionManager`. Он отвечает за обработку абстрагированных событий и на их основе
составляет сессии. Модуль `manager` содержит типы, относящиеся, как ни странно, к менеджеру сессий.

### `trait Clock`: Абстракция времени

Первое, что важно понимать в менеджере сессий и в самом тестировании `core`, — это именно этот трейт.
Он содержит только один метод `now()`:

> ```rust
> pub trait Clock {
>     fn now(&self) -> u64;
> }
> ```
>
> [Source](https://github.com/spyland-dev/spyland/blob/1465e924feab9587321f1753ce2f2a0e1b374126/core/src/manager.rs#L46-L52)

Такой трейт нужен для того, чтобы менеджер мог работать как с реальным временем, так и с имитируемым
в тестах. Например, реализация с настоящим временем:

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
> [Source](https://github.com/spyland-dev/spyland/blob/1465e924feab9587321f1753ce2f2a0e1b374126/daemon/src/main.rs#L86-L95)

Подразумевается, что `now()` возвращает [UNIX-время (Timestamp)](https://en.wikipedia.org/wiki/Unix_time),
однако формат может варьироваться от реализации.

### `struct SessionManager`: Менеджер сессий

Эта структура в качестве generic типа принимает реализацию `trait Clock`. Основной метод менеджера
это `handle_event()`. Как не трудно догадаться, эта функция принимает `Event` и обрабатывает его.
Исходя из событий, менеджер выполняет действия:

- #### `ActiveWindowChanged(Option<String>)`
Создаёт новую сессию из `app_id` и текущего `workspace`.
**Ответы**: `SessionCreated`, `Ignored` (приложение скрыто конфигурацией).
- #### `WorkspaceChanged(i32)`
Синхронизирует текущий workspace с менеджером. Обновляет приватное поле `workspace`.
**Ответы**: `Handled`.
- #### `Idle(bool)`
Если `true`, сохраняет текущую сессию и создаёт новую с состоянием `State::Idle`.
Если `false`, заканчивает `Idle` сессию и возобновляет старую.
**Ответы**: `SessionIdled(bool)`, `Handled` (создана idle сессия, не сохранена текущая), `Ignored`.
- #### `Tick`
Время (`Clock::now`) изменилось. Во время этого события, менеджер проверяет, прошло ли с момента
последнего `flush()` (автоматического) больше времени, чем указано в конфигурации.
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
> [Source](https://github.com/spyland-dev/spyland/blob/1465e924feab9587321f1753ce2f2a0e1b374126/core/src/manager.rs#L105-L120)

Менеджер сессий может конфигурироваться данной структурой. Она по большей части предназначена для
конфигов spyland, но чтобы не загружать `core`, у крейта присутствует feature `serde`, которая
добавляет (де)сериализацию у некоторых структур, включая эту.

- #### `flush_interval`
Периодичность автоматического `flush()`. Формат времени также зависит от `trait Clock`.
- #### `hidden_applications`
Вектор `app_id` приложений, который менеджер будет игнорировать.
- #### `min_session_duration`
Минимальная продолжительность сессии. Если она меньше, менеджер проигнорирует её. Формат времени
также зависит от `trait Clock`.

Методы `config()` и `set_config()` позволяют просматривать и изменять конфигурацию менеджера соответственно. По умолчанию, менеджер использует функцию `default()` у конфигурации
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
> [Source](https://github.com/spyland-dev/spyland/blob/1465e924feab9587321f1753ce2f2a0e1b374126/core/src/manager.rs#L122-L130)

### `enum Response`: Ответы

Менеджер в методах `handle_event()` и `flush()` возвращает `Response`. Этот ответ представляет
собой действие, которое он выполнил. Для подробного описания, смотрите
[документацию `handle_event()`](https://github.com/spyland-dev/spyland/blob/1465e924feab9587321f1753ce2f2a0e1b374126/core/src/manager.rs#L154-L170),
и [документацию `Response`](https://github.com/spyland-dev/spyland/blob/1465e924feab9587321f1753ce2f2a0e1b374126/core/src/manager.rs#L65-L98).

### `fn flush()`: Сохранение сессий

Менеджер текущую сессию всегда хранит в приватном поле `current`. Если вы захотите получить сессии
через метод `sessions()`, то вам обязательно придётся сохранить текущую
для получения актуальной информации.

Для избежания потери данных, в `SessionManager` существуют автоматические `flush()`, которые
вызываются при `Event::Tick`. Однако, тогда бы сессии хранились бы неэффективно, поэтому во
`flush()` есть автоматическое слияние. Если состояния текущей и последней сессий совпадают,
тогда `flush()` обновляет эту последнюю сессию, изменяя `Session::end` на текущее время.

Проверка на минимальную длительность сессии (`min_session_duration`) происходит именно в этой
функции.

### Внутренние поля `SessionManager`

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
> [Source](https://github.com/spyland-dev/spyland/blob/1465e924feab9587321f1753ce2f2a0e1b374126/core/src/manager.rs#L55-L63)

Краткое описание для каждого поле:

- **`current`**: Текущая сессия. Может быть `None`, если сессия не инициализирована.
- **`workspace`**: Текущий workspace. `None` при инициализации. Изменяется через `WorkspaceChanged`.
- **`clock`**: Экземпляр `C` (`trait Clock`).
- **`sessions`**: Сюда `flush()` сохраняет сессии. Возвращается от `sessions()`.
- **`old_session`**: Используется для сохранения сессии при `Event::Idle`.
- **`last_flush`**: Время последнего автоматического `flush()`.
- **`config`**: Конфигурация менеджера в виде структуры.

## Аналитика сессий

В `core` есть ещё один модуль - `analytics`. Единственный тип внутри — `struct SessionAnalytics`.
Структура для инициализации (`new()`) принимает вектор сессий. Она предоставляет удобную статистику
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

Каждый тест начинается одинаково — с инициализации `TestDriver`:

```rust
#[test]
fn test_name() {
    // Переменная должна быть изменяемой (mut),
    // так как состояние SessionManager будет меняться.
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
> [Source](https://github.com/spyland-dev/spyland/blob/1465e924feab9587321f1753ce2f2a0e1b374126/core/tests/lib.rs#L50-L72)

# `lib`: API-библиотека для spyland

`spyland-lib` — не менее важный крейт, чем `spyland-core`.
Он используется во всех остальных крейтах и является API-библиотекой для
внешнего взаимодействия со spyland.

## `ipc`: Inter-Process Communication с Unix-демоном

Этот модуль используется для взаимодействия с Unix-демоном `spylandd`. IPC получил своё
предназначение после PR [#1](https://github.com/spyland-dev/spyland/pull/1). На данный момент
он используется для передачи событий от бэкендов к Unix-демону `spylandd`.

Структуры в корне модуля являются очень простыми обёртками. Так, например, задача
`IpcServer` — принимать соединения (`IpcConnection`) через функцию `accept()`.
`IpcConnection` абстрагирует соединение с сервером, позволяя получать `Request` и
отправлять `Response`. `IpcClient` является клиентом для взаимодействия с Unix-демоном — отправляет
клиентский запрос (`Request`) и читает серверный ответ (`Response`).

### `mod protocol`: IPC-протокол

Подмодуль `protocol` определяет протокол общения между сервером (`spylandd`) и
клиентом. Он содержит два enum и две функции. Перечисления (enum) — это `Request` и `Response`,
которые мы видели выше. А функции — это `read()` и `send()`. Они представляют собой
простое чтение/вывод из/в UnixStream, возвращая/принимая
`serde::DeserializeOwned`/`serde::Serialize` соответственно.

Помимо этого есть ещё `u32` константа `VERSION`, которая обозначает текущую версию протокола.
Используется для рукопожатия (`Request::Handshake`).

## `db`: Работа с базой данных spyland

Второй модуль `db` содержит две структуры: `Db` — обёртка SQL-запросов для управления БД, и
`SessionSql` — представление `spyland_core::Session` в виде более удобной обёртки для работы с БД.

## `path`: Пути по умолчанию

Последний модуль предоставляет простые функции для получения путей по умолчанию, которые использует
spyland, также имея "ensure" аналоги, которые "убеждаются", что путь существует или сокет свободен.
Все функции возвращают `Result<PathBuf>`.

Также, в debug сборках, пути имеют другие имена, добавляя в имя файла суффикс `-debug`, чтобы
избежать конфликта с реальными данными потенциально установленного spyland на компьютере.

- `get_database_path()` / `ensure_database_path()`
  Путь к базе данных сессий spyland.<br>

  **Путь**: `$XDG_STATE_HOME/spyland/sessions.sqlite` или
  `$HOME/.local/state/spyland/sessions.sqlite`.<br>

  **"ensure"-поведение** (при вызове функции `ensure_database_path()`):
  Стандартное поведение как и для всех остальных функций: Создать директорию, если она отсутствует.
  Наличие файла не проверяется из-за автоматического создания sqlite.
  Используйте `Db::open_readonly()` для получения ошибки в случае, если файл не создан или в
  других ситуациях проверяйте вручную.

- `get_socket_path()` / `ensure_socket_path()`
  Путь к сокету Unix-демона `spylandd`.<br>

  **Путь**: `$XDG_RUNTIME_DIR/spyland.sock`<br>

  **"ensure"-поведение**:
  Если файл уже существует, то удалить его, тем самым освободив сокет.
  Использовать только для Unix-демона.

- `get_config_path()` / `ensure_config_path()`
  Путь к конфигурации spyland.<br>

  **Путь**: `$XDG_CONFIG_HOME/spyland/config.toml` или
  `$HOME/.config/spyland/config.toml`.<br>

  **"ensure"-поведение**:
  Если файл не существует, то создать его.
