import React from 'react';
import { Trash2, FileX, FileCheck, RefreshCw, ExternalLink } from 'lucide-react';
import { ImageFile } from '../App';
import { formatFileSize } from '../utils/fileUtils';

interface ImageListProps {
  images: ImageFile[];
  onRemoveImage: (id: string) => void;
  onRemoveAllImages: () => void;
}

const ImageList: React.FC<ImageListProps> = ({ 
  images, 
  onRemoveImage,
  onRemoveAllImages
}) => {
  const openFile = (path: string) => {
    if (path) {
      window.electron.ipcRenderer.invoke('open-file', path);
    }
  };

  if (images.length === 0) {
    return (
      <div className="h-full flex flex-col items-center justify-center text-gray-500 dark:text-gray-400">
        <FileX className="h-12 w-12 mb-2 opacity-40" />
        <p>No images added yet</p>
      </div>
    );
  }

  return (
    <div className="card h-full flex flex-col">
      <div className="flex justify-between mb-3">
        <h3 className="font-medium text-gray-700 dark:text-gray-300">
          Images ({images.length})
        </h3>
        
        {images.length > 0 && (
          <button
            onClick={onRemoveAllImages}
            className="text-sm text-red-600 hover:text-red-800 dark:text-red-400 dark:hover:text-red-300 transition-colors flex items-center"
          >
            <Trash2 className="h-3 w-3 mr-1" />
            Clear all
          </button>
        )}
      </div>
      
      <div className="overflow-auto flex-1">
        <div className="space-y-3">
          {images.map((image) => (
            <div 
              key={image.id}
              className="flex items-center p-2 rounded-md border border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800"
            >
              <div className="w-12 h-12 bg-gray-100 dark:bg-gray-700 rounded overflow-hidden mr-3 flex-shrink-0">
                <img 
                  src={image.preview} 
                  alt={image.name} 
                  className="w-full h-full object-cover"
                />
              </div>
              
              <div className="flex-1 min-w-0">
                <div className="flex justify-between">
                  <p className="text-sm font-medium text-gray-900 dark:text-white truncate">
                    {image.name}
                  </p>
                  
                  <div className="flex items-center space-x-1">
                    {image.status === 'converting' && (
                      <RefreshCw className="h-4 w-4 text-yellow-500 animate-spin" />
                    )}
                    
                    {image.status === 'converted' && (
                      <FileCheck className="h-4 w-4 text-green-500" />
                    )}
                    
                    {image.status === 'error' && (
                      <FileX className="h-4 w-4 text-red-500" />
                    )}
                    
                    <button
                      onClick={() => onRemoveImage(image.id)}
                      className="text-gray-400 hover:text-red-500 transition-colors"
                    >
                      <Trash2 className="h-4 w-4" />
                    </button>
                  </div>
                </div>
                
                <div className="flex text-xs text-gray-500 dark:text-gray-400 mt-1">
                  <span className="mr-3">{formatFileSize(image.size)}</span>
                  
                  {image.status === 'converted' && (
                    <>
                      <span className="mr-3">→ {formatFileSize(image.newSize || 0)}</span>
                      <span className="text-green-600 dark:text-green-400">
                        Saved {image.compressionRatio}%
                      </span>
                      
                      {image.outputPath && (
                        <button 
                          onClick={() => openFile(image.outputPath as string)}
                          className="ml-2 text-teal-600 hover:text-teal-800 dark:text-teal-400 flex items-center"
                        >
                          <ExternalLink className="h-3 w-3 mr-1" />
                          Open
                        </button>
                      )}
                    </>
                  )}
                  
                  {image.status === 'error' && (
                    <span className="text-red-600 dark:text-red-400">
                      {image.error || 'Conversion failed'}
                    </span>
                  )}
                </div>
              </div>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
};

export default ImageList;