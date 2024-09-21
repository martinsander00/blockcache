import usdcIcon from '../images/usdc.png';
import solanaIcon from '../images/solana.png';
import Chart from 'chart.js/auto'; // Import Chart.js
import 'chartjs-adapter-date-fns'; // Add this import for date handling
import './styles.css'; // Import CSS

// Define the VolumeResponse interface
interface VolumeResponse {
  pool_address: string;
  volume: number;
}

// Define pool addresses with corresponding icons
const pools = [
  {
    address: '7xuPLn8Bun4ZGHeD95xYLnPKReKtSe7zfVRzRJWJZVZW',
    label: 'Pool Address 1',
    icon: usdcIcon, // Use the imported USDC icon
  },
  {
    address: 'CWjGo5jkduSW5LN5rxgiQ18vGnJJEKWPCXkpJGxKSQTH',
    label: 'Pool Address 2',
    icon: solanaIcon, // Use the imported SOL icon
  },
];

// Initialize data structures
interface VolumeDataPoint {
  timestamp: Date;
  volume: number;
}

const volumeDataMap: { [address: string]: VolumeDataPoint[] } = {};
const latestPriceMap: { [address: string]: number | null } = {};
const chartsMap: { [address: string]: Chart } = {}; // Map to store chart instances

// Function to fetch volume data (using POST request as per server expectation)
async function fetchVolume(poolAddress: string): Promise<number | null> {
  try {
    const response = await fetch(`http://127.0.0.1:8000/volume`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({ pool_address: poolAddress }), // Send the pool address in the request body
    });

    if (!response.ok) {
      const errorText = await response.text();
      console.error(`Error fetching volume for ${poolAddress}: ${response.status} ${response.statusText}\n${errorText}`);
      return null;
    }

    const data: VolumeResponse = await response.json();
    return data.volume;
  } catch (error) {
    console.error(`Error fetching volume for ${poolAddress}:`, error);
    return null;
  }
}

// Create chart instances dynamically
function createChartElement(poolAddress: string): HTMLCanvasElement {
  const canvas = document.createElement('canvas');
  canvas.id = `chart-${poolAddress}`;
  canvas.width = 400;
  canvas.height = 300;

  const chartContainer = document.createElement('div');
  chartContainer.className = 'graph-container';
  chartContainer.appendChild(canvas);

  const graphSpace = document.createElement('div');
  graphSpace.className = 'graph-space';
  graphSpace.id = `graph-space-${poolAddress}`; // Add ID for easy reference
  graphSpace.appendChild(chartContainer);

  document.getElementById(`row-${poolAddress}`)?.after(graphSpace);

  const ctx = canvas.getContext('2d');
  if (!ctx) {
    throw new Error('Failed to get canvas context.');
  }

  const chart = new Chart(ctx, {
    type: 'line',
    data: {
      labels: [],
      datasets: [{
        label: '5-minute Volume',
        data: [],
        borderColor: 'rgba(75, 192, 192, 1)',
        fill: false,
      }],
    },
    options: {
      responsive: true,
      scales: {
        x: {
          type: 'time',
          time: {
            unit: 'minute',
            displayFormats: {
              minute: 'HH:mm',
            },
          },
          title: {
            display: true,
            text: 'Time',
          },
        },
        y: {
          title: {
            display: true,
            text: 'Volume',
          },
        },
      },
    },
  });

  // Store the chart instance in the map
  chartsMap[poolAddress] = chart;

  return canvas;
}

// Update chart with new data
function updateChart(poolAddress: string, dataPoint: VolumeDataPoint) {
  const chart = chartsMap[poolAddress]; // Get the chart instance from the map
  if (chart) {
    chart.data.labels?.push(dataPoint.timestamp);
    chart.data.datasets[0].data.push(dataPoint.volume);

    // Keep only the last 50 data points to prevent memory issues
    if (chart.data.labels && chart.data.labels.length > 50) {
      chart.data.labels.shift();
      chart.data.datasets[0].data.shift();
    }

    chart.update();
  }
}

// Function to update the displayed volume and arrow
function updateVolumeDisplay(poolAddress: string, newVolume: number) {
  const previousPrice = latestPriceMap[poolAddress];
  latestPriceMap[poolAddress] = newVolume;

  const volumeElement = document.getElementById(`volume-${poolAddress}`);
  if (volumeElement) {
    // Determine the arrow and color based on price movement
    let arrow = '';
    let color = 'black'; // Default to black for no change
    if (previousPrice !== null) {
      if (newVolume > previousPrice) {
        arrow = '↑'; // Price increased
        color = 'green';
      } else if (newVolume < previousPrice) {
        arrow = '↓'; // Price decreased
        color = 'red';
      }
    }

    // Update the content of the volume element with the new price and arrow
    volumeElement.textContent = `${arrow} ${newVolume.toFixed(2)}`;
    volumeElement.style.color = color;
  }
}

// Function to periodically fetch and update the volume for each pool
function startAutoUpdate() {
  setInterval(() => {
    pools.forEach(async (pool) => {
      const newVolume = await fetchVolume(pool.address);
      if (newVolume !== null) {
        updateVolumeDisplay(pool.address, newVolume);

        // Update the chart with the new volume
        const dataPoint: VolumeDataPoint = { timestamp: new Date(), volume: newVolume };
        volumeDataMap[pool.address] = volumeDataMap[pool.address] || [];
        volumeDataMap[pool.address].push(dataPoint);

        // Update the chart if it exists
        updateChart(pool.address, dataPoint);
      }
    });
  }, 10000); // 10 seconds interval
}

// Fetch prices immediately when the page loads
async function fetchInitialPrices() {
  for (const pool of pools) {
    const volume = await fetchVolume(pool.address);
    if (volume !== null) {
      updateVolumeDisplay(pool.address, volume);

      // Update the chart with the initial volume
      const dataPoint: VolumeDataPoint = { timestamp: new Date(), volume };
      volumeDataMap[pool.address] = volumeDataMap[pool.address] || [];
      volumeDataMap[pool.address].push(dataPoint);

      // Update the chart if it exists
      updateChart(pool.address, dataPoint);
    }
  }
}

// Component to render pool rows
function Pools() {
  const poolContainer = document.getElementById('pools-container');

  if (!poolContainer) {
    throw new Error('Pools container not found!');
  }

  pools.forEach((pool) => {
    const row = document.createElement('div');
    row.className = 'pool-row';
    row.id = `row-${pool.address}`;

    const icon = document.createElement('img');
    icon.src = pool.icon; // Set icon based on pool
    icon.alt = `${pool.label} icon`;
    row.appendChild(icon);

    const poolText = document.createElement('span');
    const shortenedAddress = `${pool.address.slice(0, 4)}...${pool.address.slice(-4)}`;
    poolText.textContent = shortenedAddress;
    row.appendChild(poolText);

    const volumeText = document.createElement('span');
    volumeText.id = `volume-${pool.address}`;
    volumeText.textContent = 'Loading...'; // Initially loading
    row.appendChild(volumeText);

    // Add event listener to display the graph on row click
    row.addEventListener('click', async () => {
      const graphSpace = document.getElementById(`graph-space-${pool.address}`);

      if (graphSpace) {
        // If the graph space exists, remove it when clicked again
        graphSpace.remove();
      } else {
        // Create the graph and insert below if it doesn't exist
        const canvas = createChartElement(pool.address);
        const volume = await fetchVolume(pool.address);

        if (volume !== null) {
          const dataPoint: VolumeDataPoint = { timestamp: new Date(), volume };
          volumeDataMap[pool.address] = volumeDataMap[pool.address] || [];
          volumeDataMap[pool.address].push(dataPoint);

          updateVolumeDisplay(pool.address, volume);

          // Update the chart with the initial data
          updateChart(pool.address, dataPoint);
        }
      }
    });

    poolContainer.appendChild(row);
  });

  // Add Dummy Rows
  for (let i = 0; i < 10; i++) {
    const dummyRow = document.createElement('div');
    dummyRow.className = 'pool-row';
    dummyRow.innerHTML = `
      <img src="${solanaIcon}" alt="Pool Icon" />
      <span>CUkd...DIoj${i + 1}</span>
      <span>999.99</span>
    `;
    poolContainer.appendChild(dummyRow);
  }
}

// Initialize Pool component
Pools();

// Start automatic update every 10 seconds
startAutoUpdate();

// Fetch prices immediately on page load
fetchInitialPrices();

