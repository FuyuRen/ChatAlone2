// Function to create a universal popup
function createUniversalPopup(options = {}) {
    const {
      title = "Popup",
      content = "",
      onClose = () => {},
    } = options;
  
    // Create the popup container
    const popupContainer = document.createElement('div');
    popupContainer.classList.add(
      'fixed',
      'inset-0',
      'bg-background/80',
      'backdrop-blur-sm',
      'z-50',
      'flex',
      'items-center',
      'justify-center',
      'hidden' // Initially hidden
    );
  
    // Create the popup card
    const popupCard = document.createElement('div');
    popupCard.classList.add(
      'w-full',
      'max-w-lg',
      'mx-auto',
      'p-4',
      'bg-white',
      'rounded-lg',
      'shadow-md'
    );
  
    // Create the popup header
    const popupHeader = document.createElement('div');
    popupHeader.classList.add(
      'flex',
      'items-center',
      'justify-between',
      'pb-4'
    );
  
    // Add title to the header
    const popupTitle = document.createElement('h2');
    popupTitle.classList.add('text-2xl', 'font-bold');
    popupTitle.textContent = title;
    popupHeader.appendChild(popupTitle);
  
    // Create the close button
    const closeButton = document.createElement('button');
    closeButton.classList.add(
      'p-1',
      'rounded-full',
      'hover:bg-gray-100',
      'focus:outline-none',
      'focus:ring-2',
      'focus:ring-gray-200'
    );
    closeButton.innerHTML = `
      <svg
        xmlns="http://www.w3.org/2000/svg"
        fill="none"
        viewBox="0 0 24 24"
        stroke-width="1.5"
        stroke="currentColor"
        class="w-6 h-6"
      >
        <path stroke-linecap="round" stroke-linejoin="round" d="M6 18L18 6M6 6l12 12" />
      </svg>
    `;
    closeButton.addEventListener('click', () => {
      popupContainer.classList.add('hidden');
      onClose();
    });
    popupHeader.appendChild(closeButton);
  
    // Create the popup body
    const popupBody = document.createElement('div');
    popupBody.classList.add('py-4');
    
    // Append elements to the popup
    popupCard.appendChild(popupHeader);
    popupCard.appendChild(popupBody);
    popupContainer.appendChild(popupCard);
    document.body.appendChild(popupContainer);
  
    // Function to show the popup
    function showPopup() {
      popupContainer.classList.remove('hidden');
    }
  
    // Function to hide the popup
    function hidePopup() {
      popupContainer.classList.add('hidden');
      onClose();
    }
  
    // Set the content of the popup
    if (typeof content === 'string') {
      popupBody.innerHTML = content;
    } else if (content instanceof HTMLElement) {
      popupBody.appendChild(content);
    }
  
    // Handle escape key press to close the popup
    const handleEscape = (event) => {
      if (event.key === 'Escape') {
        hidePopup();
      }
    };
  
    popupContainer.addEventListener('click', (event) => {
      // Check if the click is outside of the popup card
      if (event.target === popupContainer) {
        hidePopup();
      }
    });
  
    document.addEventListener('keydown', handleEscape);
  
    // Return functions to control the popup
    return {
      show: showPopup,
      hide: hidePopup
    };
  }