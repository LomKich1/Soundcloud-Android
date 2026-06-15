import React from 'react';
import { useLocation, useNavigate } from 'react-router-dom';

// Иконки — inline SVG, чтобы не зависеть от конкретных имён экспортов lib/icons
const HomeIcon = ({ size = 22, strokeWidth = 1.8 }: { size?: number; strokeWidth?: number }) => (
  <svg width={size} height={size} viewBox="0 0 24 24" fill="none"
    stroke="currentColor" strokeWidth={strokeWidth} strokeLinecap="round" strokeLinejoin="round">
    <path d="m3 9 9-7 9 7v11a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z" />
    <polyline points="9 22 9 12 15 12 15 22" />
  </svg>
);

const SearchIcon = ({ size = 22, strokeWidth = 1.8 }: { size?: number; strokeWidth?: number }) => (
  <svg width={size} height={size} viewBox="0 0 24 24" fill="none"
    stroke="currentColor" strokeWidth={strokeWidth} strokeLinecap="round" strokeLinejoin="round">
    <circle cx="11" cy="11" r="8" />
    <path d="m21 21-4.35-4.35" />
  </svg>
);

const LibraryIcon = ({ size = 22, strokeWidth = 1.8 }: { size?: number; strokeWidth?: number }) => (
  <svg width={size} height={size} viewBox="0 0 24 24" fill="none"
    stroke="currentColor" strokeWidth={strokeWidth} strokeLinecap="round" strokeLinejoin="round">
    <path d="M4 19.5v-15A2.5 2.5 0 0 1 6.5 2H20v20H6.5a2.5 2.5 0 0 1 0-5H20" />
  </svg>
);

const DiscoverIcon = ({ size = 22, strokeWidth = 1.8 }: { size?: number; strokeWidth?: number }) => (
  <svg width={size} height={size} viewBox="0 0 24 24" fill="none"
    stroke="currentColor" strokeWidth={strokeWidth} strokeLinecap="round" strokeLinejoin="round">
    <circle cx="12" cy="12" r="10" />
    <polygon points="16.24 7.76 14.12 14.12 7.76 16.24 9.88 9.88 16.24 7.76" />
  </svg>
);

const SettingsIcon = ({ size = 22, strokeWidth = 1.8 }: { size?: number; strokeWidth?: number }) => (
  <svg width={size} height={size} viewBox="0 0 24 24" fill="none"
    stroke="currentColor" strokeWidth={strokeWidth} strokeLinecap="round" strokeLinejoin="round">
    <path d="M12.22 2h-.44a2 2 0 0 0-2 2v.18a2 2 0 0 1-1 1.73l-.43.25a2 2 0 0 1-2 0l-.15-.08a2 2 0 0 0-2.73.73l-.22.38a2 2 0 0 0 .73 2.73l.15.1a2 2 0 0 1 1 1.72v.51a2 2 0 0 1-1 1.74l-.15.09a2 2 0 0 0-.73 2.73l.22.38a2 2 0 0 0 2.73.73l.15-.08a2 2 0 0 1 2 0l.43.25a2 2 0 0 1 1 1.73V20a2 2 0 0 0 2 2h.44a2 2 0 0 0 2-2v-.18a2 2 0 0 1 1-1.73l.43-.25a2 2 0 0 1 2 0l.15.08a2 2 0 0 0 2.73-.73l.22-.39a2 2 0 0 0-.73-2.73l-.15-.08a2 2 0 0 1-1-1.74v-.5a2 2 0 0 1 1-1.74l.15-.09a2 2 0 0 0 .73-2.73l-.22-.38a2 2 0 0 0-2.73-.73l-.15.08a2 2 0 0 1-2 0l-.43-.25a2 2 0 0 1-1-1.73V4a2 2 0 0 0-2-2z" />
    <circle cx="12" cy="12" r="3" />
  </svg>
);

interface NavItem {
  path: string;
  label: string;
  Icon: React.FC<{ size?: number; strokeWidth?: number }>;
}

const NAV_ITEMS: NavItem[] = [
  { path: '/home',     label: 'Главная', Icon: HomeIcon    },
  { path: '/search',   label: 'Поиск',   Icon: SearchIcon  },
  { path: '/library',  label: 'Библио',  Icon: LibraryIcon },
  { path: '/discover', label: 'Обзор',   Icon: DiscoverIcon},
  { path: '/settings', label: 'Настр.',  Icon: SettingsIcon},
];

/**
 * Нижняя навигационная панель для мобильных устройств.
 * Рендерится вместо боковой Sidebar на экранах < 768px.
 * Размещается НАД NowPlayingBar (z-40, bottom отступ соответствует высоте плеера ~72px).
 */
export const MobileNavBar = React.memo(() => {
  const navigate = useNavigate();
  const { pathname } = useLocation();

  return (
    <nav
      className="fixed left-0 right-0 z-40 flex items-center justify-around border-t"
      style={{
        bottom: 'env(safe-area-inset-bottom, 0px)',
        paddingBottom: 'env(safe-area-inset-bottom, 4px)',
        background: 'rgba(8, 8, 12, 0.97)',
        backdropFilter: 'blur(24px)',
        WebkitBackdropFilter: 'blur(24px)',
        borderColor: 'rgba(255,255,255,0.07)',
        // Высота плеера NowPlayingBar ≈ 72px, ставим себя прямо над ним
        marginBottom: '72px',
      }}
    >
      {NAV_ITEMS.map(({ path, label, Icon }) => {
        const active = pathname === path || pathname.startsWith(path + '/');
        return (
          <button
            key={path}
            type="button"
            onClick={() => navigate(path)}
            className="flex flex-col items-center justify-center gap-0.5 flex-1 py-2.5 transition-all duration-150 active:scale-90"
            style={{
              color: active
                ? 'var(--color-accent, #f50)'
                : 'rgba(255,255,255,0.38)',
              background: 'transparent',
              border: 'none',
              cursor: 'pointer',
              minHeight: '52px',
            }}
            aria-current={active ? 'page' : undefined}
            aria-label={label}
          >
            <Icon size={22} strokeWidth={active ? 2.4 : 1.7} />
            <span
              style={{
                fontSize: '9px',
                fontWeight: active ? 700 : 500,
                letterSpacing: '0.02em',
                marginTop: '1px',
                lineHeight: 1,
              }}
            >
              {label}
            </span>
          </button>
        );
      })}
    </nav>
  );
});

MobileNavBar.displayName = 'MobileNavBar';
