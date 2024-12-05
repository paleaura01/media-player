// renderer/library.js

export async function selectFolderOrFiles() {
  console.log("Opening file/folder dialog...");
  return await window.electron.selectFolderOrFiles();
} 

export async function readDirectory(folderPath) {
  console.log(`Reading directory: ${folderPath}`);
  const items = await window.electron.readDirectory(folderPath);
  console.log(`Items read from directory (${folderPath}):`, items);
  return items;
}

// Utility: Check if a file is an audio file
export function isAudioFile(filePath) {
  const audioExtensions = [".mp3", ".wav", ".ogg", ".opus"];
  return audioExtensions.includes(filePath.slice(filePath.lastIndexOf(".")).toLowerCase());
}
