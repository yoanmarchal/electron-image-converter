import React from 'react';
import { Folder, Sliders, Play } from 'lucide-react';
import { ConversionSettings as Settings } from '../App';

interface ConversionSettingsProps {
  settings: Settings;
  onSettingsChange: (settings: Partial<Settings>) => void;
  onSelectOutputDir: () => Promise<void>;
  onConvert: () => Promise<void>;
  isConverting: boolean;
  imageCount: number;
}

const ConversionSettings: React.FC<ConversionSettingsProps> = ({
  settings,
  onSettingsChange,
  onSelectOutputDir,
  onConvert,
  isConverting,
  imageCount,
}) => {
  const handleQualityChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const quality = parseInt(e.target.value, 10);
    onSettingsChange({ quality });
  };

  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-lg font-semibold flex items-center text-gray-900 dark:text-white">
          <Sliders className="h-5 w-5 mr-2 text-teal-600" />
          Conversion Settings
        </h2>
      </div>
      
      <div className="space-y-4">
        <div>
          <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
            WebP Quality: {settings.quality}%
          </label>
          <div className="flex items-center">
            <span className="text-xs text-gray-500 mr-2">Low</span>
            <input
              type="range"
              min="0"
              max="100"
              value={settings.quality}
              onChange={handleQualityChange}
              className="w-full h-2 bg-gray-200 rounded-lg appearance-none cursor-pointer accent-teal-600 dark:bg-gray-700"
              disabled={isConverting}
            />
            <span className="text-xs text-gray-500 ml-2">High</span>
          </div>
          <div className="text-xs text-gray-500 mt-1">
            {settings.quality < 30 ? 'Low quality, smaller file size' : 
             settings.quality < 70 ? 'Balanced quality and size' : 
             'High quality, larger file size'}
          </div>
        </div>
        
        <div>
          <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
            Output Directory
          </label>
          <div className="flex">
            <div className="flex-1 overflow-hidden">
              <div className="truncate text-sm p-2 border border-gray-300 dark:border-gray-700 rounded-l-md bg-gray-50 dark:bg-gray-800 h-full">
                {settings.outputDir || 'No directory selected'}
              </div>
            </div>
            <button
              type="button"
              onClick={onSelectOutputDir}
              className="flex items-center justify-center px-3 py-2 border border-gray-300 dark:border-gray-700 rounded-r-md bg-gray-100 hover:bg-gray-200 dark:bg-gray-700 dark:hover:bg-gray-600 transition-colors"
              disabled={isConverting}
            >
              <Folder className="h-4 w-4 text-gray-700 dark:text-gray-300" />
            </button>
          </div>
        </div>
      </div>
      
      <div className="pt-4 border-t border-gray-200 dark:border-gray-700">
        <button
          type="button"
          onClick={onConvert}
          className="btn btn-primary w-full flex items-center justify-center group"
          disabled={isConverting || imageCount === 0 || !settings.outputDir}
        >
          {isConverting ? (
            <>
              <svg className="animate-spin -ml-1 mr-2 h-4 w-4 text-white" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
                <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4"></circle>
                <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
              </svg>
              Converting...
            </>
          ) : (
            <>
              <Play className="h-4 w-4 mr-2 group-hover:animate-pulse" />
              Convert to WebP
            </>
          )}
        </button>
        
        {(imageCount === 0 || !settings.outputDir) && !isConverting && (
          <p className="mt-2 text-xs text-gray-500 dark:text-gray-400">
            {imageCount === 0 ? 'Please add images to convert' : 'Please select an output directory'}
          </p>
        )}
      </div>
    </div>
  );
};

export default ConversionSettings;