import React from 'react';
import { Trash2, FileText, ExternalLink, Calendar } from 'lucide-react';
import { HistoryItem } from '../App';
import { formatFileSize, formatPercent, getFilenameFromPath } from '../utils/fileUtils';

interface ConversionHistoryProps {
  history: HistoryItem[];
  onClearHistory: () => Promise<void>;
}

const dateFormatter = new Intl.DateTimeFormat('fr-FR', {
  dateStyle: 'medium',
  timeStyle: 'short',
});

const ConversionHistory: React.FC<ConversionHistoryProps> = ({
  history,
  onClearHistory
}) => {
  const openFile = (path: string) => {
    if (path) {
      window.electron.ipcRenderer.invoke('open-file', path);
    }
  };

  if (history.length === 0) {
    return (
      <div className="h-full flex flex-col items-center justify-center text-gray-500 dark:text-gray-400">
        <FileText className="h-12 w-12 mb-2 opacity-40" aria-hidden="true" />
        <p>Aucune conversion pour le moment</p>
      </div>
    );
  }

  return (
    <div className="p-4 h-full overflow-auto">
      <div className="flex justify-between items-center mb-4">
        <h2 className="text-xl font-semibold text-gray-900 dark:text-white">
          Historique des conversions
        </h2>

        <button
          type="button"
          onClick={onClearHistory}
          className="btn btn-danger flex items-center"
        >
          <Trash2 className="h-4 w-4 mr-2" aria-hidden="true" />
          Vider l'historique
        </button>
      </div>

      <div className="bg-white dark:bg-gray-800 rounded-lg shadow overflow-hidden">
        <table className="min-w-full divide-y divide-gray-200 dark:divide-gray-700">
          <thead className="bg-gray-50 dark:bg-gray-900">
            <tr>
              <th scope="col" className="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">
                Fichier
              </th>
              <th scope="col" className="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">
                Taille d'origine
              </th>
              <th scope="col" className="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">
                Taille convertie
              </th>
              <th scope="col" className="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">
                Gain
              </th>
              <th scope="col" className="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">
                Date
              </th>
              <th scope="col" className="px-6 py-3 text-right text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">
                Actions
              </th>
            </tr>
          </thead>
          <tbody className="bg-white dark:bg-gray-800 divide-y divide-gray-200 dark:divide-gray-700">
            {history.map((item) => {
              const fileName = getFilenameFromPath(item.originalPath);
              const outputName = getFilenameFromPath(item.outputPath);
              const isSmaller = parseFloat(item.compressionRatio) >= 0;

              return (
                <tr key={item.id} className="hover:bg-gray-50 dark:hover:bg-gray-750 transition-colors">
                  <td className="px-6 py-4 whitespace-nowrap">
                    <div className="text-sm font-medium text-gray-900 dark:text-white truncate max-w-xs" title={item.originalPath}>
                      {fileName}
                    </div>
                    <div className="text-xs text-gray-500 dark:text-gray-400 truncate max-w-xs" title={item.outputPath}>
                      → {outputName}
                    </div>
                  </td>
                  <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500 dark:text-gray-400">
                    {formatFileSize(item.originalSize)}
                  </td>
                  <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500 dark:text-gray-400">
                    {formatFileSize(item.newSize)}
                  </td>
                  <td className="px-6 py-4 whitespace-nowrap">
                    <span className={`px-2 inline-flex text-xs leading-5 font-semibold rounded-full ${
                      isSmaller
                        ? 'bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200'
                        : 'bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-200'
                    }`}>
                      {formatPercent(item.compressionRatio)}
                    </span>
                  </td>
                  <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500 dark:text-gray-400">
                    <div className="flex items-center">
                      <Calendar className="h-3 w-3 mr-1" aria-hidden="true" />
                      {dateFormatter.format(new Date(item.timestamp))}
                    </div>
                  </td>
                  <td className="px-6 py-4 whitespace-nowrap text-right text-sm font-medium">
                    <button
                      type="button"
                      onClick={() => openFile(item.outputPath)}
                      className="text-teal-600 hover:text-teal-900 dark:text-teal-400 dark:hover:text-teal-300 ml-4 flex items-center"
                      aria-label={`Ouvrir ${outputName}`}
                    >
                      <ExternalLink className="h-4 w-4 mr-1" aria-hidden="true" />
                      Ouvrir
                    </button>
                  </td>
                </tr>
              );
            })}
          </tbody>
        </table>
      </div>
    </div>
  );
};

export default ConversionHistory;
