<div align="center">

[English](../en/CONTRIBUTING.md) | **Русский**

</div>

Перед тем, как создать Pull Request в мой проект, прочитайте это руководство.

Спасибо за интерес к `spyland`!

# Код

## Стиль кода

Для форматирования используйте [`rustfmt`](https://github.com/rust-lang/rustfmt).
Это может быть напрямую через `cargo fmt`, или через ваш LSP.

### Импорты (`use`)

В этом проекте нет строгих правил для форматирования импортов.
Вы встретите разные стили:

**Развёрнутый стиль:**
```rust
use spyland_core::Clock;
use spyland_core::Configuration;
use spyland_core::Event;
use spyland_core::SessionManager;
use std::cell::RefCell;
use std::rc::Rc;
```

**Структурированный стиль:**
```rust
use std::{
    io::{BufRead, BufReader, Write},
    os::unix::net::{UnixListener, UnixStream},
    sync::{
        Arc, Mutex,
        mpsc::{self, Receiver, Sender},
    },
    thread,
    time::Duration,
};
```

**Смешанный стиль:**
```rust
use log::{error, warn};
use niri_ipc::socket::Socket;
use niri_ipc::{Event as NiriEvent, Request, Response};
use spyland_core::{Backend, Event};
use std::path::PathBuf;
use std::sync::mpsc;
```

Форматируйте `use` импорты, как вы посчитаете нужным, но удовлетворяя `rustfmt`.

## Тесты

**Это критически важно!** `spyland` разрабатывается с требованием,
что весь новый код должен быть протестирован.

К каждому новому функционалу требуются тесты.
Также как и баг фиксы: исправляете баг — будьте добры написать тест к нему.

Если вы считаете, что в вашем случае они не нужны откройте issue
или включите в PR обоснование.

### Константы

Рассматривая код тестов, вы можете обнаружить частое использование констант.
Для кода тестов у нас есть правило: если вы используете одно и тоже значение
времени компиляции несколько раз (например для валидации данных), то объявите
константу внутри функции и используйте именно её, вместо одинаковых значений.
Это сделано для корректности данных и обозначения взаимосвязи.

**Неправильно**, не используется константы — "волшебные" значения:
```rust
d.event(Event::WorkspaceChanged(2));
d.event(Event::ActiveWindowChanged(Some("discord".into())));
d.flush();

match &d.mgr.sessions()[0].state {
    State::Active { app_id, workspace } => {
        assert_eq!("discord", app_id, "app_id not matching");
        assert_eq!(
            2,
            workspace.expect("workspace is none"),
            "workspace not matching"
        );
    }
    _ => panic!("Incorrect state"),
}
```
**Неправильно**, используется всего один раз — нет взаимосвязи:
```rust
const APP_ID: &str = "firefox";

d.event(Event::ActiveWindowChanged(Some(APP_ID.into())));
d.flush();

assert_eq!(d.mgr.sessions().len(), 1, "Less then one sessions");
```
**Правильно**, используется константы, есть взаимосвязь:
```rust
const WORKSPACE: i32 = 1;
const APP_ID: &str = "firefox";

d.event(Event::WorkspaceChanged(WORKSPACE));
d.event(Event::ActiveWindowChanged(Some(APP_ID.into())));
d.flush();

match &d.mgr.sessions()[0].state {
    State::Active { app_id, workspace } => {
        assert_eq!(APP_ID, app_id, "app_id not matching");
        assert_eq!(
            WORKSPACE,
            workspace.expect("workspace is none"),
            "workspace not matching"
        );
    }
    _ => panic!("Incorrect state"),
}
```

## Искусственный интеллект

Использование ИИ разрешено, но под вашим строгим контролем. Вы должны:
- Полностью понимать сгенерированный код
- Проверить, что код работает корректно
- Взять на себя ответственность за качество

Если мы обнаружим, что код явно сгенерирован ИИ без понимания — PR может быть отклонён.

# Коммиты

Названия коммитов должны следовать
[Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/):

```
type(scope): description
```

# Документация

## Комментарии в коде (`//`)

Комментарии должны быть говорящими, а не очевидными.

Правильный пример:
```rust
let mut d = TestDriver::new();

d.event(Event::ActiveWindowChanged(Some("firefox".into())));
d.tick(30);
// d.update_and_flush();
// not needed because of automatic update()
// and update() in SessionManager
```

Неправильный пример:
```rust
let mut d = TestDriver::new();

d.event(Event::ActiveWindowChanged(Some("firefox".into())));
d.update_and_flush(); // explicit flushes
```

Для выделения важных комментариев используйте префиксы:
```rust
// TODO: optimize this loop
// FIXME: handle edge case when buffer is empty
// NOTE: this must run before db initialization
// WARN: never call this from async context
```

## Документация API (`///`, `//!`)

Документируйте публичный API крейтов `spyland-lib` и `spyland-core`.

Требования:
1. Краткое описание
2. Более подробное описание (если нужно)
3. Описание параметров функции в разделе `# Arguments` (если функция)
4. Примеры использования в разделе `# Example`
5. Предупреждения в разделе `# Panics` или `# Safety` (при необходимости)
6. Doctests для проверки примеров

## Markdown документация (`docs/`)

Файлы в `docs/` должны существовать на двух языках:
- `docs/en/` — Английская версия
- `docs/ru/` — Русская версия

В корне репозитория находятся символические ссылки на английскую версию.

При изменении документации:
1. Обновите основной английский язык
2. Проверьте корректность всех ссылок (внутренние, внешние, картинки)
3. Убедитесь, что перевод точен
4. Проверьте форматирование (заголовки, код, списки)

---

Если у вас остались вопросы:
1. Опирайтесь на существующий материал (код, документация) в репозитории
2. Откройте issue с вопросом
3. Обсудите в PR перед началом работы

Спасибо за помощь в развитии `spyland`!
