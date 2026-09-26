import React, { useCallback, useState } from 'react';
import { useDropzone, DropEvent } from 'react-dropzone';
import { ImageDown, Upload } from 'lucide-react';
import { ImageFile } from '../App';

interface DropZoneProps {
  onFilesSelected: (files: ImageFile[]) => void;
  isConverting: boolean;
  className?: string;
}

interface SelectedFile {
  path: string;
  previewUrl: string;
}

interface ImageInfo {
  format: string;
  width: number;
  height: number;
  size: number;
}

/**
 * Renvoie les File natifs de l'événement. Par défaut, react-dropzone (file-selector) passe par
 * getAsFileSystemHandle().getFile(), dont les File n'ont pas de chemin sur le disque :
 * webUtils.getPathForFile renverrait alors une chaîne vide.
 */
async function getNativeFilesFromEvent(event: DropEvent): Promise<Array<File | DataTransferItem>> {
  if (Array.isArray(event)) return [];

  if ('dataTransfer' in event && event.dataTransfer) {
    // Pendant le survol, les fichiers ne sont pas encore lisibles : seuls les items servent à valider le type
    return event.type === 'drop'
      ? Array.from(event.dataTransfer.files)
      : Array.from(event.dataTransfer.items).filter(item => item.kind === 'file');
  }

  const input = event.target as HTMLInputElement | null;
  return input?.files ? Array.from(input.files) : [];
}

const DropZone: React.FC<DropZoneProps> = ({ onFilesSelected, isConverting, className = '' }) => {
  const [isDragging, setIsDragging] = useState(false);
  const [isProcessing, setIsProcessing] = useState(false);

  const onDrop = useCallback(async (acceptedFiles: File[]) => {
    if (acceptedFiles.length === 0) return;
    
    setIsProcessing(true);
    try {
      // Traiter tous les fichiers en parallèle
      const processedFiles = await Promise.all(
        acceptedFiles.map(async (file) => {
          try {
            // Utiliser le chemin d'origine pour que la sortie soit écrite à côté du fichier source
            const filePath = window.electron.getPathForFile(file);
            if (!filePath) {
              console.error('Dropped file has no path on disk:', file.name);
              return null;
            }

            const info = await window.electron.ipcRenderer.invoke<ImageInfo | null>('get-image-info', filePath);

            if (info) {
              return {
                id: crypto.randomUUID(),
                name: file.name,
                path: filePath,
                size: info.size,
                preview: URL.createObjectURL(file),
                status: 'pending',
              };
            }
            return null;
          } catch (error) {
            console.error('Error processing file:', file.name, error);
            return null;
          }
        })
      );
      
      const validFiles = processedFiles.filter(Boolean) as ImageFile[];
      if (validFiles.length > 0) {
        onFilesSelected(validFiles);
      }
    } finally {
      setIsProcessing(false);
    }
  }, [onFilesSelected]);

  const handleSelectFiles = async () => {
    try {
      const selectedFiles = await window.electron.ipcRenderer.invoke('select-files') as SelectedFile[];
      
      if (selectedFiles && selectedFiles.length > 0) {
        const imageFiles: ImageFile[] = [];
        
        for (const file of selectedFiles) {
          const info = await window.electron.ipcRenderer.invoke<ImageInfo | null>('get-image-info', file.path);
          
          if (info) {
            imageFiles.push({
              id: crypto.randomUUID(),
              name: file.path.split('/').pop() || file.path.split('\\').pop() || 'unknown',
              path: file.path,
              size: info.size,
              preview: file.previewUrl,
              status: 'pending',
            });
          }
        }
        
        onFilesSelected(imageFiles);
      }
    } catch (error) {
      console.error('Failed to select files:', error);
    }
  };

  const { getRootProps, getInputProps, isDragActive } = useDropzone({
    onDrop,
    accept: {
      'image/*': ['.jpg', '.jpeg', '.png', '.gif', '.tif', '.tiff', '.webp', '.avif']
    },
    disabled: isConverting,
    getFilesFromEvent: getNativeFilesFromEvent,
    // Utiliser l'<input type="file"> natif au clic plutôt que showOpenFilePicker (File sans chemin)
    useFsAccessApi: false,
  });

  // Update isDragging state based on isDragActive
  React.useEffect(() => {
    setIsDragging(isDragActive);
  }, [isDragActive]);

  return (
    <div 
      {...getRootProps()} 
      className={`card border-2 border-dashed transition-all duration-300 flex flex-col items-center justify-center py-8 cursor-pointer ${
        isDragging 
          ? 'border-teal-500 bg-teal-50 dark:bg-teal-900/20' 
          : 'border-gray-300 dark:border-gray-600 hover:border-teal-400 dark:hover:border-teal-700'
      } ${(isConverting || isProcessing) ? 'opacity-50 pointer-events-none' : ''} ${className}`}
    >
      <input {...getInputProps()} />
      
      <div className={`p-4 rounded-full bg-teal-100 dark:bg-teal-900/50 mb-4 transition-transform duration-300 ${
        isDragging ? 'scale-110' : ''
      }`}>
        {isProcessing ? (
          <div className="animate-spin">
            <Upload className="h-10 w-10 text-teal-600 dark:text-teal-400" />
          </div>
        ) : (
          <Upload className={`h-10 w-10 text-teal-600 dark:text-teal-400 transition-transform duration-300 ${
            isDragging ? 'rotate-12' : ''
          }`} />
        )}
      </div>
      
      <div className="text-center px-4">
        <p className="text-lg font-medium text-gray-700 dark:text-gray-300 mb-2">
          {isProcessing ? 'Processing images...' :
           isDragging ? 'Drop images here' : 'Drag and drop images here'}
        </p>
        <p className="text-sm text-gray-500 dark:text-gray-400 mb-4">
          Or click to select files
        </p>
        
        <button
          type="button"
          onClick={(e) => {
            e.stopPropagation();
            handleSelectFiles();
          }}
          className="btn btn-primary"
          disabled={isConverting}
        >
          <ImageDown className="h-4 w-4 mr-2 inline" />
          Select Images
        </button>
      </div>
    </div>
  );
};

export default DropZone;