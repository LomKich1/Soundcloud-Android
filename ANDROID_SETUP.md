# Настройка Android-сборки

## 1. Генерация Keystore (один раз)

Выполни на своём компьютере:

```bash
keytool -genkey -v \
  -keystore release.keystore \
  -alias soundcloud \
  -keyalg RSA -keysize 2048 \
  -validity 10000
```

Программа задаст несколько вопросов — имя, организация и т.д.  
Всё можно заполнить произвольно. **Пароли запомни — они понадобятся.**

## 2. Добавление секретов в GitHub

Зайди в репозиторий → **Settings → Secrets and variables → Actions → New repository secret**

Добавь четыре секрета:

| Имя секрета | Значение |
|---|---|
| `ANDROID_KEYSTORE_BASE64` | base64 от файла keystore (команда ниже) |
| `ANDROID_KEY_ALIAS` | `soundcloud` (или другой alias, что задал) |
| `ANDROID_KEYSTORE_PASSWORD` | пароль, введённый при genkey |
| `ANDROID_KEY_PASSWORD` | пароль ключа (обычно тот же) |

**Как получить base64 от keystore:**

```bash
# macOS / Linux
base64 -i release.keystore | pbcopy   # скопирует в буфер
# или
base64 -i release.keystore > keystore_b64.txt   # сохранит в файл

# Windows (PowerShell)
[Convert]::ToBase64String([IO.File]::ReadAllBytes("release.keystore")) | Set-Clipboard
```

## 3. Запуск сборки

После добавления секретов сборка запустится автоматически при каждом push.  
APK появится в **Actions → последний workflow run → Artifacts → soundcloud-android-apk**.

Если хочешь собрать вручную без пуша:  
Actions → **Build Android APK** → **Run workflow**.

## 4. Создание релиза с APK

Создай тег вида `v1.0.0`:

```bash
git tag v1.0.0
git push origin v1.0.0
```

GitHub автоматически создаст Release с APK-файлом во вкладке **Releases**.

## Что работает на Android

- ✅ Воспроизведение треков (streaming)
- ✅ Авторизация SoundCloud
- ✅ Кэширование треков (в исходном формате, без перекодирования в m4a)
- ✅ DPI-bypass (для обхода блокировок)
- ✅ Кастомные обои, изображения
- ✅ Поиск и стриминг

## Что недоступно на Android

- ❌ Discord Rich Presence (нет IPC на Android)
- ❌ Системные медиа-контролы в шторке (требует отдельной реализации через Android MediaSession)
- ❌ Перекодирование в AAC (нет ffmpeg subprocess)
- ❌ Системный трей / mini-player окно
- ❌ Single-instance защита
