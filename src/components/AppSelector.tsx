import React, { useState, useEffect } from 'react';
import './AppSelector.css';

interface AppInfo {
  id: string;
  name: string;
  bundleId: string;
  icon: string;
  path: string;
}

interface AppSelectorProps {
  isVisible: boolean;
  onClose: () => void;
  onSelectApp: (app: AppInfo) => void;
}

const AppSelector: React.FC<AppSelectorProps> = ({
  isVisible,
  onClose,
  onSelectApp
}) => {
  const [searchQuery, setSearchQuery] = useState('');
  const [apps, setApps] = useState<AppInfo[]>([]);
  const [filteredApps, setFilteredApps] = useState<AppInfo[]>([]);

  // 模拟应用列表 - 在实际应用中应该从系统获取
  const mockApps: AppInfo[] = [
    {
      id: '1',
      name: 'AirPort Utility',
      bundleId: 'com.apple.airport.airportutility',
      icon: '📡',
      path: '/Applications/Utilities/AirPort Utility.app'
    },
    {
      id: '2',
      name: 'App Store',
      bundleId: 'com.apple.AppStore',
      icon: '🛍️',
      path: '/Applications/App Store.app'
    },
    {
      id: '3',
      name: 'Audio MIDI Setup',
      bundleId: 'com.apple.audio.AudioMIDISetup',
      icon: '🎹',
      path: '/Applications/Utilities/Audio MIDI Setup.app'
    },
    {
      id: '4',
      name: 'Automator',
      bundleId: 'com.apple.Automator',
      icon: '🤖',
      path: '/Applications/Automator.app'
    },
    {
      id: '5',
      name: 'BaiduNetdisk',
      bundleId: 'com.baidu.netdisk',
      icon: '☁️',
      path: '/Applications/BaiduNetdisk.app'
    },
    {
      id: '6',
      name: 'Bluetooth File Exchange',
      bundleId: 'com.apple.BluetoothFileExchange',
      icon: '📲',
      path: '/Applications/Utilities/Bluetooth File Exchange.app'
    },
    {
      id: '7',
      name: 'Books',
      bundleId: 'com.apple.iBooksX',
      icon: '📚',
      path: '/Applications/Books.app'
    },
    {
      id: '8',
      name: 'Boot Camp Assistant',
      bundleId: 'com.apple.bootcampassistant',
      icon: '💾',
      path: '/Applications/Utilities/Boot Camp Assistant.app'
    },
    {
      id: '9',
      name: 'Calculator',
      bundleId: 'com.apple.calculator',
      icon: '🧮',
      path: '/Applications/Calculator.app'
    },
    {
      id: '10',
      name: 'Calendar',
      bundleId: 'com.apple.iCal',
      icon: '📅',
      path: '/Applications/Calendar.app'
    },
    {
      id: '11',
      name: 'ChatGPT',
      bundleId: 'com.openai.chatgpt',
      icon: '🤖',
      path: '/Applications/ChatGPT.app'
    },
    {
      id: '12',
      name: 'Chrome',
      bundleId: 'com.google.Chrome',
      icon: '🌐',
      path: '/Applications/Google Chrome.app'
    },
    {
      id: '13',
      name: 'CleanMyMac X',
      bundleId: 'com.macpaw.CleanMyMac',
      icon: '🧹',
      path: '/Applications/CleanMyMac X.app'
    },
    {
      id: '14',
      name: 'Console',
      bundleId: 'com.apple.Console',
      icon: '📋',
      path: '/Applications/Utilities/Console.app'
    },
    {
      id: '15',
      name: 'Contacts',
      bundleId: 'com.apple.AddressBook',
      icon: '👥',
      path: '/Applications/Contacts.app'
    },
    {
      id: '16',
      name: 'Discord',
      bundleId: 'com.hnc.Discord',
      icon: '💬',
      path: '/Applications/Discord.app'
    },
    {
      id: '17',
      name: 'Disk Utility',
      bundleId: 'com.apple.DiskUtility',
      icon: '💿',
      path: '/Applications/Utilities/Disk Utility.app'
    },
    {
      id: '18',
      name: 'FaceTime',
      bundleId: 'com.apple.FaceTime',
      icon: '📹',
      path: '/Applications/FaceTime.app'
    },
    {
      id: '19',
      name: 'Finder',
      bundleId: 'com.apple.finder',
      icon: '🔍',
      path: '/System/Library/CoreServices/Finder.app'
    },
    {
      id: '20',
      name: 'Firefox',
      bundleId: 'org.mozilla.firefox',
      icon: '🦊',
      path: '/Applications/Firefox.app'
    },
    {
      id: '21',
      name: 'GitHub Desktop',
      bundleId: 'com.github.GitHubClient',
      icon: '🐙',
      path: '/Applications/GitHub Desktop.app'
    },
    {
      id: '22',
      name: 'Mail',
      bundleId: 'com.apple.mail',
      icon: '📧',
      path: '/Applications/Mail.app'
    },
    {
      id: '23',
      name: 'Messages',
      bundleId: 'com.apple.MobileSMS',
      icon: '💬',
      path: '/Applications/Messages.app'
    },
    {
      id: '24',
      name: 'Music',
      bundleId: 'com.apple.Music',
      icon: '🎵',
      path: '/Applications/Music.app'
    },
    {
      id: '25',
      name: 'Notes',
      bundleId: 'com.apple.Notes',
      icon: '📝',
      path: '/Applications/Notes.app'
    },
    {
      id: '26',
      name: 'Photos',
      bundleId: 'com.apple.Photos',
      icon: '🖼️',
      path: '/Applications/Photos.app'
    },
    {
      id: '27',
      name: 'Preview',
      bundleId: 'com.apple.Preview',
      icon: '👁️',
      path: '/Applications/Preview.app'
    },
    {
      id: '28',
      name: 'Reminders',
      bundleId: 'com.apple.reminders',
      icon: '✅',
      path: '/Applications/Reminders.app'
    },
    {
      id: '29',
      name: 'Safari',
      bundleId: 'com.apple.Safari',
      icon: '🧭',
      path: '/Applications/Safari.app'
    },
    {
      id: '30',
      name: 'Slack',
      bundleId: 'com.tinyspeck.slackmacgap',
      icon: '💼',
      path: '/Applications/Slack.app'
    },
    {
      id: '31',
      name: 'Spotify',
      bundleId: 'com.spotify.client',
      icon: '🎧',
      path: '/Applications/Spotify.app'
    },
    {
      id: '32',
      name: 'System Preferences',
      bundleId: 'com.apple.systempreferences',
      icon: '⚙️',
      path: '/Applications/System Preferences.app'
    },
    {
      id: '33',
      name: 'Terminal',
      bundleId: 'com.apple.Terminal',
      icon: '💻',
      path: '/Applications/Utilities/Terminal.app'
    },
    {
      id: '34',
      name: 'TextEdit',
      bundleId: 'com.apple.TextEdit',
      icon: '📄',
      path: '/Applications/TextEdit.app'
    },
    {
      id: '35',
      name: 'Visual Studio Code',
      bundleId: 'com.microsoft.VSCode',
      icon: '💙',
      path: '/Applications/Visual Studio Code.app'
    },
    {
      id: '36',
      name: 'WhatsApp',
      bundleId: 'net.whatsapp.WhatsApp',
      icon: '💬',
      path: '/Applications/WhatsApp.app'
    },
    {
      id: '37',
      name: 'Xcode',
      bundleId: 'com.apple.dt.Xcode',
      icon: '🔨',
      path: '/Applications/Xcode.app'
    },
    {
      id: '38',
      name: 'YouTube',
      bundleId: 'com.google.Chrome.app.kjgfgldnnfoeklkmfkjfagphfepbbdan',
      icon: '📺',
      path: '/Applications/YouTube.app'
    },
    {
      id: '39',
      name: 'Zoom',
      bundleId: 'us.zoom.xos',
      icon: '📹',
      path: '/Applications/zoom.us.app'
    }
  ];

  useEffect(() => {
    setApps(mockApps);
    setFilteredApps(mockApps);
  }, []);

  useEffect(() => {
    if (searchQuery) {
      const filtered = apps.filter(app => 
        app.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
        app.bundleId.toLowerCase().includes(searchQuery.toLowerCase())
      );
      setFilteredApps(filtered);
    } else {
      setFilteredApps(apps);
    }
  }, [searchQuery, apps]);

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Escape') {
      onClose();
    }
  };

  if (!isVisible) return null;

  return (
    <div className="app-selector-overlay" onClick={onClose}>
      <div className="app-selector-dialog" onClick={(e) => e.stopPropagation()}>
        <div className="app-selector-header">
          <h2>选择应用程序</h2>
          <button className="close-btn" onClick={onClose}>
            <span>Esc</span>
            <span className="close-icon">✕</span>
          </button>
        </div>

        <div className="app-selector-search">
          <span className="search-icon">🔍</span>
          <input
            type="text"
            placeholder="Search"
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            onKeyDown={handleKeyDown}
            autoFocus
          />
        </div>

        <div className="app-selector-list">
          {filteredApps.map((app) => (
            <div
              key={app.id}
              className="app-item"
              onClick={() => {
                onSelectApp(app);
                onClose();
              }}
            >
              <div className="app-icon">{app.icon}</div>
              <div className="app-info">
                <div className="app-name">{app.name}</div>
                <div className="app-bundle">{app.bundleId}</div>
              </div>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
};

export default AppSelector;