// Select DOM elements
const greeting = document.getElementById('greeting') as HTMLElement;
const actionBtn = document.getElementById('actionBtn') as HTMLButtonElement;
const poolAddressInput = document.getElementById('poolAddressInput') as HTMLInputElement;
const responseSection = document.getElementById('responseSection') as HTMLElement;

// Add event listener to the button
actionBtn.addEventListener('click', async () => {
  const poolAddress = poolAddressInput.value.trim();

  if (poolAddress === '') {
    responseSection.textContent = 'Please enter a Pool Address.';
    return;
  }

  // Update greeting to indicate loading
  greeting.textContent = 'Fetching volume...';

  // Create the request payload
  const payload = {
    pool_address: poolAddress,
  };

  try {
    const response = await fetch('http://127.0.0.1:8000/volume', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify(payload),
    });

    if (!response.ok) {
      const errorText = await response.text();
      responseSection.textContent = `Error: ${response.status} ${response.statusText}\n${errorText}`;
      greeting.textContent = 'Welcome to BlockCache';
      return;
    }

    const data: VolumeResponse = await response.json();

    // Display the response
    responseSection.innerHTML = `
      <p><strong>Pool Address:</strong> ${data.pool_address}</p>
      <p><strong>5-minute Volume:</strong> ${data.volume}</p>
    `;
  } catch (error) {
    console.error('Error fetching volume:', error);
    responseSection.textContent = 'An error occurred while fetching the volume.';
  } finally {
    // Reset greeting
    greeting.textContent = 'Welcome to BlockCache';
  }
});

// Define interfaces matching the backend structures
interface VolumeResponse {
  pool_address: string;
  volume: number;
}

