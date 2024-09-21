// place files you want to import through the `$lib` alias in this folder.

/**
 * Performs a fetch request to the specified API endpoint
 * @param {string} url - The API endpoint URL
 * @param {Object} options - Request options (method, headers, body, etc.)
 * @returns {Promise<any>} - The parsed JSON response
 */
export async function fetchApi(url, options = {}) {
  try {
    const response = await fetch(url, {
      method: 'GET',
      headers: {
        'Content-Type': 'application/json',
        // Add any other headers here
      },
      ...options,
    });

    if (!response.ok) {
      throw new Error(`HTTP error! status: ${response.status}`);
    }

    return await response.json();
  } catch (error) {
    console.error('Fetch error:', error);
    throw error;
  }
}

// Usage example:
// import { fetchApi } from '$lib';
// 
// async function getData() {
//   try {
//     const data = await fetchApi('https://api.example.com/data');
//     console.log(data);
//   } catch (error) {
//     console.error('Error fetching data:', error);
//   }
// }