# Инструкция по применению патча

## Диагностика: Почему был чёрный экран

Найдено **4 независимые причины**, каждая из которых достаточна для чёрного экрана:

### Причина 1 — Rust: `call-client` не защищён на Android  
В `lib.rs` строки:
```rust
let call_state = network::call::CallState::init(data_dir.clone(), rt_handle);
network::call::manage_state(app.handle(), call_state.clone());
network::call::maybe_autostart(app.handle(), call_state);  // ← может зависать!
```
`maybe_autostart` на Android пытается запустить нативный процесс или открыть
привилегированный сокет → зависание setup-колбэка → WebView никогда не открывается.

### Причина 2 — Rust: `call_set_enabled` / `call_status` без `CallState` в registry  
Если setup всё же завершился, JS вызывал бы `invoke('call_set_enabled')` →
Tauri пытается извлечь `State<CallState>` → **panic**, потому что CallState
не был зарегистрирован через `app.manage()` на Android.

### Причина 3 — Frontend: Sidebar занимает всю ширину экрана  
`AppShell` всегда рендерил `<Sidebar />` (≈250px) рядом с `<main class="flex-1">`.
На телефоне 360px: `main` = 360 − 250 = 110px. Контент уходил за экран и не был виден.
На узких устройствах `overflow-hidden` на родителе полностью скрывал контент.

### Причина 4 — Frontend + Android WebView: нет фона у `<body>`  
`<body>` не имел `background-color`. Android WebView с `decorations: false`
даёт прозрачный фон → нативный чёрный фон приложения просвечивал, пока
React монтировался (0.3–1 с) → пользователь видел чёрный экран даже если
React в итоге рендерился.

### Причина 5 (вероятная) — AndroidManifest: нет `usesCleartextTraffic`  
warp HTTP-серверы слушают `http://127.0.0.1:PORT`. Android 9+ (API 28+)
блокирует cleartext HTTP к любым хостам (даже localhost) без явного флага.
Прокси-запросы к SoundCloud через warp не проходили → контент не загружался.

---

## Что изменено

| Файл | Изменение |
|------|-----------|
| `desktop/src-tauri/src/lib.rs` | `call-client` и `dpi_desync` завёрнуты в `#[cfg(not(target_os = "android"))]`; `call_*` и `import_*` команды исключены из `invoke_handler!` на Android |
| `desktop/src-tauri/capabilities/default.json` | Добавлен `"platforms": ["linux","macos","windows"]` — файл применяется только на десктопе |
| `desktop/src-tauri/capabilities/android.json` | **Новый файл** — capabilities только для Android без window-controls |
| `desktop/index.html` | Добавлен `background-color: #060609` на `body`/`html`/`#root`; mobile viewport с `viewport-fit=cover`; убраны tap-задержки |
| `desktop/src/components/layout/AppShell.tsx` | Добавлен хук `useIsMobile()` (matchMedia < 768px); `<Titlebar>` и `<Sidebar>` скрыты на мобиле; `<MobileNavBar>` показывается на мобиле |
| `desktop/src/components/layout/Titlebar.tsx` | `getCurrentWindow()` в `useMemo` с try-catch; `win?.method()` с optional chaining |
| `desktop/src/components/layout/MobileNavBar.tsx` | **Новый файл** — нижняя навигационная панель для Android |
| `.github/workflows/android.yml` | Добавлен шаг `Patch AndroidManifest.xml` (добавляет `usesCleartextTraffic="true"` и проверяет INTERNET permission) |

---

## Пошаговое применение

### Шаг 1: Скопируй файлы в нужные места

```
Скопируй из этого архива → в свой репозиторий:

fix-output/desktop/src-tauri/src/lib.rs
    → desktop/src-tauri/src/lib.rs          (ЗАМЕНИТЬ)

fix-output/desktop/src-tauri/capabilities/default.json
    → desktop/src-tauri/capabilities/default.json   (ЗАМЕНИТЬ)

fix-output/desktop/src-tauri/capabilities/android.json
    → desktop/src-tauri/capabilities/android.json   (НОВЫЙ ФАЙЛ)

fix-output/desktop/index.html
    → desktop/index.html                    (ЗАМЕНИТЬ)

fix-output/desktop/src/components/layout/AppShell.tsx
    → desktop/src/components/layout/AppShell.tsx    (ЗАМЕНИТЬ)

fix-output/desktop/src/components/layout/Titlebar.tsx
    → desktop/src/components/layout/Titlebar.tsx    (ЗАМЕНИТЬ)

fix-output/desktop/src/components/layout/MobileNavBar.tsx
    → desktop/src/components/layout/MobileNavBar.tsx  (НОВЫЙ ФАЙЛ)

fix-output/.github/workflows/android.yml
    → .github/workflows/android.yml         (ЗАМЕНИТЬ)
```

### Шаг 2: Проверь импорты в MobileNavBar.tsx

В `MobileNavBar.tsx` используются inline SVG-иконки (без зависимости от `lib/icons`).
Если в твоём проекте есть другие пути навигации (не /home, /search, /library, /discover, /settings),
отредактируй массив `NAV_ITEMS` в `MobileNavBar.tsx`.

### Шаг 3: Закоммить и запустить GitHub Actions

```bash
git add .
git commit -m "fix: adapt for Android — mobile layout, cfg guards, cleartext HTTP"
git push
```

GitHub Actions запустит сборку автоматически.

### Шаг 4: Проверь логи сборки

В Actions найди шаг `Patch AndroidManifest.xml`. Убедись что в выводе видно:
```
usesCleartextTraffic=true добавлен
✅ INTERNET permission: OK
```

---

## Если после патча всё ещё что-то не работает

### Проблема: контент страниц не загружается (пустые списки, спиннер)
**Причина**: warp-прокси работает, но SoundCloud отдаёт 403 (невалидная сессия).  
**Решение**: Нужно залогиниться — QR-код на экране логина показывает ссылку.

### Проблема: навигация (Home/Search/etc.) не работает
**Причина**: роуты в `NAV_ITEMS` не совпадают с твоими роутами.  
**Решение**: Проверь пути в `src/App.tsx` и обнови `NAV_ITEMS` в `MobileNavBar.tsx`.

### Проблема: Sidebar пропал даже на планшете
**Причина**: `useIsMobile()` проверяет `< 768px`.  
**Решение**: Измени порог в `AppShell.tsx`: `window.innerWidth < 1024` для планшетов.

### Проблема: аудио не воспроизводится
**Причина**: `cpal` на Android требует разрешения `MODIFY_AUDIO_SETTINGS`.  
**Решение**: Добавь в `AndroidManifest.xml` через шаг патча:
```xml
<uses-permission android:name="android.permission.MODIFY_AUDIO_SETTINGS" />
```

### Проблема: `cargo tauri android build` падает с ошибкой компиляции
Возможно, `dpi-desync` или `call-client` не компилируются для Android.
Добавь в `Cargo.toml` в секцию `[target.'cfg(not(target_os = "android"))'.dependencies]`:
```toml
dpi-desync = { path = "../../utils/dpi-desync" }
call-client = { path = "../../utils/call/client" }
decrypt-client = { path = "../../utils/decrypt-client" }
```
И удали их из общего `[dependencies]`.
