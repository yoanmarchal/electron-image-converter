import React from 'react';
import { ImageDown, History, Moon, Sun } from 'lucide-react';
import classNames from 'classnames';

interface HeaderProps {
  activeTab: 'convert' | 'history';
  onTabChange: (tab: 'convert' | 'history') => void;
}

const Header: React.FC<HeaderProps> = ({ activeTab, onTabChange }) => {
  const [isDarkMode, setIsDarkMode] = React.useState(() => {
    const darkMode = localStorage.getItem('darkMode');
    const initialDarkMode = darkMode === 'true';
    if (initialDarkMode) {
      document.documentElement.classList.add('dark');
    }
    return initialDarkMode;
  });
  
  const toggleDarkMode = () => {
    const newDarkMode = !isDarkMode;
    document.documentElement.classList.toggle('dark');
    localStorage.setItem('darkMode', String(newDarkMode));
    setIsDarkMode(newDarkMode);
  };

  return (
    <header className="bg-white dark:bg-gray-800 shadow-sm">
      <div className="mx-auto px-4 sm:px-6 lg:px-8">
        <div className="flex justify-between h-16">
          <div className="flex items-center">
            <div className="flex-shrink-0 flex items-center">
              <ImageDown className="h-8 w-8 text-teal-500" />
              <span className="ml-2 text-xl font-semibold text-gray-900 dark:text-white">
                WebP Converter
              </span>
            </div>
            
            <nav className="ml-10 flex space-x-4">
              <button
                onClick={() => onTabChange('convert')}
                className={classNames(
                  'px-3 py-2 rounded-md text-sm font-medium transition-colors',
                  activeTab === 'convert'
                    ? 'bg-teal-100 text-teal-700 dark:bg-teal-900 dark:text-teal-100'
                    : 'text-gray-500 hover:bg-gray-100 dark:text-gray-300 dark:hover:bg-gray-700'
                )}
              >
                <div className="flex items-center">
                  <ImageDown className="h-4 w-4 mr-1" />
                  Convert
                </div>
              </button>
              
              <button
                onClick={() => onTabChange('history')}
                className={classNames(
                  'px-3 py-2 rounded-md text-sm font-medium transition-colors',
                  activeTab === 'history'
                    ? 'bg-teal-100 text-teal-700 dark:bg-teal-900 dark:text-teal-100'
                    : 'text-gray-500 hover:bg-gray-100 dark:text-gray-300 dark:hover:bg-gray-700'
                )}
              >
                <div className="flex items-center">
                  <History className="h-4 w-4 mr-1" />
                  History
                </div>
              </button>
            </nav>
          </div>
          
          <div className="flex items-center">
            <button
              onClick={toggleDarkMode}
              className="p-2 rounded-full text-gray-500 hover:text-gray-900 dark:text-gray-400 dark:hover:text-white focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-teal-500 transition-colors"
            >
              {isDarkMode ? (
                <Sun className="h-5 w-5" />
              ) : (
                <Moon className="h-5 w-5" />
              )}
            </button>
          </div>
        </div>
      </div>
    </header>
  );
};

export default Header;