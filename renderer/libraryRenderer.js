// renderer/libraryRenderer.js

import { readDirectory, isAudioFile } from './library.js';
import { loadTrack, playTrack } from './player.js';

export async function renderLibraryTree() {
  const libraryContainer = document.getElementById('library-tree-container');
  const libraryTree = document.getElementById('library-tree');

  try {
    const selectedFolders = await window.electron.selectFolderOrFiles();
    if (!selectedFolders || selectedFolders.length === 0) {
      libraryTree.textContent = 'No folder selected.';
      return;
    }

    libraryContainer.classList.remove('hidden');
    libraryTree.innerHTML = ''; // Clear the current tree

    const items = await readDirectory(selectedFolders[0]);

    items.forEach((item) => {
      const node = document.createElement('div');
      node.classList.add(item.type === 'directory' ? 'folder-node' : 'file-node');
      node.textContent = item.name;
      if (item.type === 'file') {
        node.setAttribute("draggable", "true"); // Make files draggable
        node.dataset.path = item.path; // Store file path for drag-and-drop
      }

      if (item.type === 'directory') {
        node.addEventListener('click', async (e) => {
          e.stopPropagation();
          const subItems = await readDirectory(item.path);
          const subTree = createSubTree(subItems);
          node.appendChild(subTree);
        });
      }

      if (item.type === 'file' && isAudioFile(item.path)) {
        node.addEventListener('click', (e) => {
          e.stopPropagation();
          loadTrack(item.path);
          playTrack();
        });
      }

      libraryTree.appendChild(node);
    });

    console.log('Library tree rendered successfully.');
  } catch (error) {
    console.error('Error rendering library tree:', error);
    libraryTree.textContent = 'Failed to load library.';
  }
}

function createSubTree(items) {
  const subTree = document.createElement('div');
  subTree.classList.add('sub-tree');

  if (items.length === 0) {
    const emptyMessage = document.createElement('div');
    emptyMessage.textContent = 'Folder is empty.';
    subTree.appendChild(emptyMessage);
    return subTree;
  }

  items.forEach((item) => {
    const subNode = document.createElement('div');
    subNode.classList.add(item.type === 'directory' ? 'folder-node' : 'file-node');
    subNode.textContent = item.name;

    if (item.type === 'file') {
      subNode.setAttribute("draggable", "true"); // Make files draggable
      subNode.dataset.path = item.path; // Store file path for drag-and-drop
    }

    if (item.type === 'directory') {
      subNode.addEventListener('click', async (e) => {
        e.stopPropagation();
        const subItems = await readDirectory(item.path);
        const subSubTree = createSubTree(subItems);
        subNode.appendChild(subSubTree);
      });
    }

    subTree.appendChild(subNode);
  });

  return subTree;
}

