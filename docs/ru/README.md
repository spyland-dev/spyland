<div align="center">

[English](../en/README.md) | **Русский**

# spyland

**Отслеживание экранного времени для Wayland**

</div>

`spyland` — проект на Rust для отслеживания экранного времени на Wayland-композиторах.
Он включает Unix-демон, CLI и библиотеку с публичным API.

## Особенности

- **Локальное хранилище**: Все данные хранятся локально в `$XDG_STATE_HOME/spyland/sessions.sqlite`
- **Минимальное потребление**: < 1% ЦП, ~8 МБ ОЗУ (spylandd)
- **Расширяемый**: Архитектура на основе backend-ов позволяет добавлять поддержку новых композиторов
- **Полностью на Rust**: Безопасность памяти и высокая производительность
- **Тестируемый**: В проекте есть автоматические тесты

## Установка

### Из исходников

Требуется Rust (последняя стабильная версия). Установите через `cargo`:

```bash
git clone https://github.com/spyland-dev/spyland
cd spyland
cargo install --path ./crates/daemon
cargo install --path ./crates/cli
```

### Nix / NixOS

Flake проекта экспортирует `packages`, `apps` и `nixosModules`. Чтобы
попробовать CLI или демон без установки, выполните:

```bash
nix run github:spyland-dev/spyland#cli
nix run github:spyland-dev/spyland#daemon
```

Для императивной установки (вместо декларативной настройки через модуль
NixOS) используйте:

```bash
nix profile add github:spyland-dev/spyland#cli
nix profile add github:spyland-dev/spyland#daemon
```

В NixOS добавьте flake в `inputs` и подключите его модуль в списке
`modules` вашей конфигурации:

```nix
{
  inputs.spyland.url = "github:spyland-dev/spyland";

  outputs = inputs@{ nixpkgs, ... }: {
    nixosConfigurations.default = nixpkgs.lib.nixosSystem {
      system = "x86_64-linux";
      modules = [
        inputs.spyland.nixosModules.default
        ./configuration.nix
      ];
    };
  };
}
```

В конфигурации NixOS включите CLI и демон. Если вы используете композитор
niri, также включите его бэкэнд:

```nix
{
  programs.spyland.enable = true;
  services.spylandd.enable = true;
  services.spylandd.backends.niri.enable = true; # только для композитора niri
}
```

## Структура проекта

```
.
├── crates                                # Крейты с исходным кодом
│   ├── cli                               # CLI-программа для взаимодействия с данными
│   ├── core                              # Ядро проекта: абстракция сессий, событий
│   ├── daemon                            # Unix-демон, отслеживающий время в фоне
│   ├── lib                               # Публичная API библиотека: БД API, IPC, утилиты
│   └── niri                              # Бэкэнд для композитора niri
│
├── docs                                  # Документация проекта
│   ├── en                                # Английский язык
│   └── ru                                # Русский язык
│
├── res                                   # Вспомогательные файлы
│   ├── nix                               # Файлы конфигурации Nix
│   │   ├── crane.nix                     # Сборка пакетов, проверки и приложения
│   │   ├── dev-shell.nix                 # Среда разработки
│   │   ├── module.nix                    # NixOS-модуль
│   │   └── outputs.nix                   # Выходы flake
│   └── services                          # Systemd-сервисы
│       ├── spyland-backend-niri.service  # Systemd-сервис для бэкэнда niri
│       ├── spyland-backends.target       # Systemd-target бэкэндов
│       └── spylandd.service              # Systemd-сервис демона
│
├── Cargo.lock
├── Cargo.toml                            # Cargo workspace
├── CONTRIBUTING.md -> docs/en/CONTRIBUTING.md
├── flake.lock
├── flake.nix                             # Точка входа Nix flake
├── LICENSE                               # Лицензия кода
└── README.md -> docs/en/README.md
```

## Дорожная карта

- [ ] **Установка**
  - [ ] Публикация крейтов на [crates.io](https://crates.io), включая бинарные приложения
  - [ ] Пакетные менеджеры
    - [ ] AUR
      - [ ] `spyland-git`
      - [ ] `spyland-bin`
  - [x] ~~Пользовательские службы systemd~~
  - [x] ~~Nix~~ [#5](https://github.com/spyland-dev/spyland/pull/5)
- [ ] **Новые backends**
  - [ ] Hyprland
  - [ ] KDE
  - [ ] Sway
  - [ ] *Mutter?*
- [ ] **UI приложение на Gtk**
- [ ] **Шифрование БД**: Защита данных пользователя
- [ ] **Проверка целостности БД**: Валидация данных при загрузке
- [x] ~~**Активности**: Группирование сессий (работа, развлечение, учёба)~~
- [x] ~~**Runtime подгрузка бэкэндов**: Динамическая загрузка бэкэндов без перекомпиляции~~ [#1](https://github.com/spyland-dev/spyland/pull/1)

## Лицензия

Проект лицензирован под GNU GPL v3.0. Подробнее в [LICENSE](../../LICENSE).
