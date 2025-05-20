import React from 'react';
import { Info, Loader } from 'lucide-react';

interface StatusBarProps {
  imageCount: number;
  convertedCount: number;
  errorCount: number;
  isConverting: boolean;
}

const StatusBar: React.FC<StatusBarProps> = ({
  imageCount,
  convertedCount,
  errorCount,
  isConverting,
}) => {
  return (
    <div className="bg-gray-100 dark:bg-gray-850 border-t border-gray-200 dark:border-gray-700 px-4 py-2 text-xs text-gray-600 dark:text-gray-400 flex items-center justify-between">
      <div className="flex items-center">
        <Info className="h-3 w-3 mr-2" />
        <span>
          {imageCount} images • {convertedCount} converted
          {errorCount > 0 && ` • ${errorCount} failed`}
        </span>
      </div>
      
      {isConverting && (
        <div className="flex items-center">
          <Loader className="h-3 w-3 mr-2 animate-spin" />
          <span>Converting images...</span>
        </div>
      )}
    </div>
  );
};

export default StatusBar;