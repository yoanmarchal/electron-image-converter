import React, { useCallback, useState } from 'react';
import { useDropzone } from 'react-dropzone';
import { ImageDown, Upload } from 'lucide-react';
import { v4 as uuidv4 } from 'uuid';
import { ImageFile } from '../App';

interface DropZoneProps {
  onFilesSelected: (files: ImageFile[]) => void;
  isConverting: boolean;
}

interface SelectedFile {
  path: string;
  previewUrl: string;
}

const DropZone: React.FC<DropZoneProps> = ({ onFilesSelected, isConverting }) => {
  const [isDragging, setIsDragging] = useState(false);

  const onDrop = useCallback(async (acceptedFiles: File[]) => {
    const imageFiles: ImageFile[] = [];
    
    for (const file of acceptedFiles) {
      try {
        // Créer un URL pour la prévisualisation
        const preview = URL.createObjectURL(file);
        
        // Lire le fichier comme un ArrayBuffer
        const buffer = await new Promise<ArrayBuffer>((resolve, reject) => {
          const reader = new FileReader();
          reader.onload = () => resolve(reader.result as ArrayBuffer);
          reader.onerror = reject;
          reader.readAsArrayBuffer(file);
        });

        // Envoyer le buffer au processus principal pour créer un fichier temporaire
        const tempPath = await window.electron.ipcRenderer.invoke('handle-dropped-file', {
          buffer: Array.from(new Uint8Array(buffer)),
          name: file.name
        });

        // Obtenir les informations sur l'image
        const info = await window.electron.ipcRenderer.invoke('get-image-info', tempPath);
        
        if (info) {
          imageFiles.push({
            id: uuidv4(),
            name: file.name,
            path: tempPath,
            size: info.size,
            preview: preview,
            status: 'pending',
          });
        }
      } catch (error) {
        console.error('Error processing file:', file.name, error);
      }
    }
    
    if (imageFiles.length > 0) {
      onFilesSelected(imageFiles);
    }
  }, [onFilesSelected]);

  const handleSelectFiles = async () => {
    try {
      const selectedFiles = await window.electron.ipcRenderer.invoke('select-files') as SelectedFile[];
      
      if (selectedFiles && selectedFiles.length > 0) {
        const imageFiles: ImageFile[] = [];
        
        for (const file of selectedFiles) {
          const info = await window.electron.ipcRenderer.invoke('get-image-info', file.path);
          
          if (info) {
            imageFiles.push({
              id: uuidv4(),
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
      'image/*': ['.jpg', '.jpeg', '.png', '.gif', '.bmp', '.tiff']
    },
    disabled: isConverting,
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
      } ${isConverting ? 'opacity-50 pointer-events-none' : ''}`}
    >
      <input {...getInputProps()} />
      
      <div className={`p-4 rounded-full bg-teal-100 dark:bg-teal-900/50 mb-4 transition-transform duration-300 ${
        isDragging ? 'scale-110' : ''
      }`}>
        <Upload className={`h-10 w-10 text-teal-600 dark:text-teal-400 transition-transform duration-300 ${
          isDragging ? 'rotate-12' : ''
        }`} />
      </div>
      
      <div className="text-center px-4">
        <p className="text-lg font-medium text-gray-700 dark:text-gray-300 mb-2">
          {isDragging ? 'Drop images here' : 'Drag and drop images here'}
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