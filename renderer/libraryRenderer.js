// renderer/libraryRenderer.js

import { readDirectory, isAudioFile } from "./library.js";
import { loadTrack, playTrack } from "./player.js";

export async function renderLibraryTree() {
  const libraryContainer = document.getElementById("library-tree-container");
  const libraryTree = document.getElementById("library-tree");

  try {
    // Select a folder to display
    const selectedFolders = await window.electron.selectFolderOrFiles();
    console.log("Selected folders:", selectedFolders);

    if (!selectedFolders || selectedFolders.length === 0) {
      libraryTree.textContent = "No folder selected.";
      return;
    }

    // Unhide library container
    libraryContainer.classList.remove("hidden");
    libraryTree.innerHTML = ""; // Clear existing content

    // Read and render directory contents
    const items = await readDirectory(selectedFolders[0].path);
    console.log(`Items in selected folder (${selectedFolders[0].path}):`, items);

    if (items.length === 0) {
      libraryTree.textContent = "Folder is empty.";
      console.log("Selected folder is empty.");
      return;
    }

    items.forEach((item) => {
      const node = document.createElement("div");
      node.classList.add(item.type === "directory" ? "folder-node" : "file-node");
      node.textContent = item.name;
    
      // Add a unique data attribute for debugging
      node.setAttribute("data-file-path", item.path);
    
      // Directory: Expand on click
      if (item.type === "directory") {
        node.addEventListener("click", async (e) => {
          e.stopPropagation(); // Prevent parent nodes from triggering
          console.log(`Expanding folder: ${item.name}`);
          const subItems = await readDirectory(item.path);
          const subTree = renderSubTree(subItems);
          node.appendChild(subTree);
        });
      } else if (item.type === "file" && isAudioFile(item.path)) {
        // File: Play the selected file
        node.addEventListener("click", (e) => {
          e.stopPropagation();
          console.log(`Playing audio file: ${item.path}`);
          loadTrack(item.path);
          playTrack();
        });
      }
    
      libraryTree.appendChild(node);
    });
    
    

    console.log("Library tree successfully rendered.");
  } catch (error) {
    console.error("Error rendering library tree:", error);
    libraryTree.textContent = "Failed to load library.";
  }
}

function renderSubTree(items) {
  const subTree = document.createElement("div");
  subTree.classList.add("sub-tree");

  if (items.length === 0) {
    const emptyMessage = document.createElement("div");
    emptyMessage.textContent = "Folder is empty.";
    subTree.appendChild(emptyMessage);
    return subTree;
  }

  items.forEach((item) => {
    const subNode = document.createElement("div");
    subNode.classList.add(item.type === "directory" ? "folder-node" : "file-node");
    subNode.textContent = item.name;

    // Directory: Expand on click
    if (item.type === "directory") {
      subNode.addEventListener("click", async () => {
        console.log(`Expanding folder: ${item.name}`);
        const subItems = await readDirectory(item.path);
        const subSubTree = renderSubTree(subItems);
        subNode.appendChild(subSubTree);
      });
    }

    // File: Play audio if it's an MP3
    if (item.type === "file" && isAudioFile(item.path)) {
      subNode.addEventListener("click", () => {
        console.log(`Playing audio file: ${item.name}`);
        loadTrack(item.path);
        playTrack();
      });
    }

    subTree.appendChild(subNode);
  });

  return subTree;
}
