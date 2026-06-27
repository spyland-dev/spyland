<div align="center">

[English](../en/README.md) | **Русский**

# spyland

**Отслеживание экранного времени для Wayland**

</div>

`spyland` — проект для отслеживания экранного времени на Wayland-композиторах,
написанный на Rust. Проект включает Unix-демон, CLI и публичную API библиотеку.

## Особенности

- **Локальное хранилище**: Все данные хранятся локально в `$XDG_STATE_HOME/spyland/sessions.sqlite`
- **Минимальное потребление**: < 1% ЦП, ~8 МБ ОЗУ (spylandd)
- **Расширяемый**: Архитектура на основе backend-ов позволяет добавлять поддержку новых композиторов
- **Полностью на Rust**: Безопасность памяти и высокая производительность
- **Тестируемый**: Весь код покрыт автоматическими тестами

## Установка

### Из исходников

Требуется Rust (последняя стабильная версия). Установите через `cargo`:

```bash
git clone https://github.com/NonExistPlayer/spyland
cd spyland
cargo install --path ./daemon
cargo install --path ./cli
```

## Структура проекта

```
.
├── crates                            # Крейты; исходный код
│   ├── cli                           # CLI программа для взаимодействия с данными
│   ├── core                          # Ядро проекта: абстракция сессий, событий
│   ├── daemon                        # Unix-демон, отслеживающий время в фоне 
│   ├── lib                           # Публичная API библиотека: БД API, IPC, утилиты
│   └── niri                          # Бэкэнд для композитора niri
│
├── docs                              # Документация проекта
│   ├── en                            # Английский язык
│   └── ru                            # Русский язык
│
├── res                               # Вспомогательные файлы (такие как сервисы)
│   ├── spyland-backend-niri.service  # Systemd-сервис для niri бэкэнда
│   ├── spyland-backends.target       # Systemd-target бэкэндов
│   └── spylandd.service              # Systemd-сервис для Unix-демона
│
├── Cargo.lock
├── Cargo.toml                        # Cargo workspace
├── CONTRIBUTING.md -> docs/en/CONTRIBUTING.md
├── flake.lock
├── flake.nix                         # Nix flake с dev-shell
├── LICENSE                           # Лицензия кода
└── README.md -> docs/en/README.md
```

## Дорожная карта

- [ ] **Активности**: Группирование сессий (работа, развлечение, учёба)
- [ ] **Установка**
  - [ ] Публикация крейтов на [crates.io](https://crates.io). Включая бинарники.
  - [ ] Пакетные менеджеры
    - [ ] AUR
      - [ ] `spyland-git`
      - [ ] `spyland-bin`
  - [ ] Системные сервисы для демона
    - [ ] systemd
- [ ] **Новые backends**
  - [ ] Hyprland
  - [ ] KDE
  - [ ] Sway
  - [ ] *Mutter?*
- [ ] **Шифрование БД**: Защита данных пользователя
- [ ] **UI приложение на Gtk**
- [ ] **Проверка целостности БД**: Валидация данных при загрузке
- [ ] **Расширенная поддержка ОС**
  - [ ] Windows
  - [ ] Android
    - [ ] Бэкэнд
    - [ ] Приложение

- [x] ~~**Runtime подгрузка бэкэндов**: Динамическая загрузка бэкэндов без перекомпиляции~~ [#1](https://github.com/NonExistPlayer/spyland/pull/1)

## Лицензия

Проект лицензирован под GNU GPL v3.0. Подробнее в [LICENSE](../../LICENSE).
