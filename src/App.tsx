import React, { useState, useEffect } from 'react';
import Header from './components/Header';
import DropZone from './components/DropZone';
import ConversionSettings from './components/ConversionSettings';
import ImageList from './components/ImageList';
import ConversionHistory from './components/ConversionHistory';
import StatusBar from './components/StatusBar';

// Declare the electron window object
declare global {
  interface Window {
    electron: {
      ipcRenderer: {
        invoke(channel: string, ...args: any[]): Promise<any>;
      };
    };
  }
}

// Define TypeScript interfaces
export interface ImageFile {
  id: string;
  name: string;
  path: string;
  size: number;
  preview: string;
  status: 'pending' | 'converting' | 'converted' | 'error';
  error?: string;
  originalSize?: number;
  newSize?: number;
  compressionRatio?: string;
  outputPath?: string;
}

export interface ConversionSettings {
  quality: number;
  outputDir: string;
  format: 'webp' | 'jpg' | 'png' | 'avif';
}

export interface HistoryItem {
  id: string;
  originalPath: string;
  outputPath: string;
  originalSize: number;
  newSize: number;
  compressionRatio: string;
  timestamp: string;
}

function App() {
  const [activeTab, setActiveTab] = useState<'convert' | 'history'>('convert');
  const [images, setImages] = useState<ImageFile[]>([]);
  const [settings, setSettings] = useState<ConversionSettings>({
    quality: 80,
    outputDir: '', // Dossier de sortie vide par défaut
    format: 'webp',
  });
  const [isConverting, setIsConverting] = useState(false);
  const [conversionHistory, setConversionHistory] = useState<HistoryItem[]>([]);

  // Load conversion history
  useEffect(() => {
    async function loadConversionHistory() {
      try {
        const history = await window.electron.ipcRenderer.invoke('get-conversion-history');
        setConversionHistory(history || []);
      } catch (error) {
        console.error('Failed to load conversion history:', error);
      }
    }

    loadConversionHistory();
  }, []);

  const handleFilesSelected = (files: ImageFile[]) => {
    setImages(prev => [...prev, ...files]);
  };

  const handleRemoveImage = (id: string) => {
    setImages(prev => prev.filter(img => img.id !== id));
  };

  const handleRemoveAllImages = () => {
    setImages([]);
  };

  const handleSettingsChange = (newSettings: Partial<ConversionSettings>) => {
    setSettings(prev => ({ ...prev, ...newSettings }));
  };

  const handleSelectOutputDir = async () => {
    try {
      const dir = await window.electron.ipcRenderer.invoke('select-output-dir');
      if (dir) {
        setSettings(prev => ({ ...prev, outputDir: dir }));
      }
    } catch (error) {
      console.error('Failed to select output directory:', error);
    }
  };

  const handleConvertImages = async () => {
    if (images.length === 0) {
      alert('Veuillez ajouter au moins une image');
      return;
    }

    setIsConverting(true);

    const pendingImages = images.filter(img => img.status === 'pending');

    for (const image of pendingImages) {
      // Update status to converting
      setImages(prev =>
        prev.map(img =>
          img.id === image.id
            ? { ...img, status: 'converting' }
            : img
        )
      );

      try {
        const result = await window.electron.ipcRenderer.invoke('convert-image', {
          filePath: image.path,
          outputDir: settings.outputDir,
          quality: settings.quality,
          format: settings.format,
        });

        if (result.success) {
          // Update with conversion results
          setImages(prev =>
            prev.map(img =>
              img.id === image.id
                ? {
                    ...img,
                    status: 'converted',
                    originalSize: result.originalSize,
                    newSize: result.newSize,
                    compressionRatio: result.compressionRatio,
                    outputPath: result.outputPath,
                  }
                : img
            )
          );

          // Add to history
          const historyItem = {
            id: image.id,
            originalPath: result.originalPath,
            outputPath: result.outputPath,
            originalSize: result.originalSize,
            newSize: result.newSize,
            compressionRatio: result.compressionRatio,
            timestamp: new Date().toISOString(),
          };

          await window.electron.ipcRenderer.invoke('save-conversion-history', historyItem);
          setConversionHistory(prev => [historyItem, ...prev]);
        } else {
          // Update with error
          setImages(prev =>
            prev.map(img =>
              img.id === image.id
                ? { ...img, status: 'error', error: result.error }
                : img
            )
          );
        }
      } catch (error) {
        console.error('Conversion error:', error);
        // Update with error
        setImages(prev =>
          prev.map(img =>
            img.id === image.id
              ? { ...img, status: 'error', error: 'Unknown error occurred' }
              : img
          )
        );
      }
    }

    setIsConverting(false);
  };

  const handleClearHistory = async () => {
    if (confirm('Are you sure you want to clear the conversion history?')) {
      await window.electron.ipcRenderer.invoke('clear-conversion-history');
      setConversionHistory([]);
    }
  };

  return (
    <div className="flex flex-col h-screen overflow-hidden">
      <Header
        activeTab={activeTab}
        onTabChange={setActiveTab}
      />

      <main className="flex-1 overflow-hidden">
        {activeTab === 'convert' ? (
          <div className="flex flex-col h-full">
            <div className="grid grid-cols-1 md:grid-cols-3 gap-4 p-4 h-full min-h-0">
              <div className="md:col-span-2 flex flex-col min-h-0">
                <DropZone
                  onFilesSelected={handleFilesSelected}
                  isConverting={isConverting}
                  className="flex-shrink-0"
                />

                <div className="mt-4 flex-1 min-h-0">
                  <ImageList
                    images={images}
                    onRemoveImage={handleRemoveImage}
                    onRemoveAllImages={handleRemoveAllImages}
                  />
                </div>
              </div>

              <div className="h-fit overflow-y-auto">
                <div className="card sticky top-0">
                  <ConversionSettings
                    settings={settings}
                    onSettingsChange={handleSettingsChange}
                    onSelectOutputDir={handleSelectOutputDir}
                    onConvert={handleConvertImages}
                    isConverting={isConverting}
                    imageCount={images.length}
                  />
                </div>
              </div>
            </div>
          </div>
        ) : (
          <ConversionHistory
            history={conversionHistory}
            onClearHistory={handleClearHistory}
          />
        )}
      </main>

      <StatusBar
        imageCount={images.length}
        convertedCount={images.filter(img => img.status === 'converted').length}
        errorCount={images.filter(img => img.status === 'error').length}
        isConverting={isConverting}
      />
    </div>
  );
}

export default App;

export { ConversionSettings }