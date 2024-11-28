// renderer/library.js

export async function selectFolderOrFiles() {
    console.log("Opening file/folder dialog...");
    return await window.electron.selectFolderOrFiles();
  }
  
  export function readDirectory(folderPath) {
    console.log(`Reading directory: ${folderPath}`);
    return window.electron.readDirectory(folderPath);
  }
  
  