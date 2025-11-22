import { Ollama } from "ollama/browser";
import { fetch as tauriFetch, ResponseType } from "@tauri-apps/api/http";
export { Ollama }

/**
 * Compress and resize an image to optimize for vision model processing
 * @param {File} file - The image file to compress
 * @param {number} maxWidth - Maximum width in pixels (default: 1024)
 * @param {number} maxHeight - Maximum height in pixels (default: 1024)
 * @param {number} quality - JPEG quality 0-1 (default: 0.8)
 * @returns {Promise<{base64: string, mediaType: string, thumbnail: string}>}
 */
export async function compressImage(file, maxWidth = 1024, maxHeight = 1024, quality = 0.8) {
  return new Promise((resolve, reject) => {
    const reader = new FileReader();

    reader.onload = (e) => {
      if (!e.target || !e.target.result) {
        reject(new Error('Failed to read file'));
        return;
      }

      const img = new Image();

      img.onload = () => {
        // Calculate new dimensions while maintaining aspect ratio
        let width = img.width;
        let height = img.height;

        if (width > maxWidth || height > maxHeight) {
          const aspectRatio = width / height;

          if (width > height) {
            width = maxWidth;
            height = width / aspectRatio;
          } else {
            height = maxHeight;
            width = height * aspectRatio;
          }
        }

        // Create canvas for full-size compressed image
        const canvas = document.createElement('canvas');
        canvas.width = width;
        canvas.height = height;
        const ctx = canvas.getContext('2d');

        if (!ctx) {
          reject(new Error('Failed to get canvas context'));
          return;
        }

        // Use better image smoothing
        ctx.imageSmoothingEnabled = true;
        ctx.imageSmoothingQuality = 'high';
        ctx.drawImage(img, 0, 0, width, height);

        // Create thumbnail (200px max)
        const thumbCanvas = document.createElement('canvas');
        const thumbMaxSize = 200;
        let thumbWidth = width;
        let thumbHeight = height;

        if (thumbWidth > thumbMaxSize || thumbHeight > thumbMaxSize) {
          const thumbRatio = thumbWidth / thumbHeight;
          if (thumbWidth > thumbHeight) {
            thumbWidth = thumbMaxSize;
            thumbHeight = thumbWidth / thumbRatio;
          } else {
            thumbHeight = thumbMaxSize;
            thumbWidth = thumbHeight * thumbRatio;
          }
        }

        thumbCanvas.width = thumbWidth;
        thumbCanvas.height = thumbHeight;
        const thumbCtx = thumbCanvas.getContext('2d');

        if (!thumbCtx) {
          reject(new Error('Failed to get thumbnail canvas context'));
          return;
        }

        thumbCtx.imageSmoothingEnabled = true;
        thumbCtx.imageSmoothingQuality = 'high';
        thumbCtx.drawImage(img, 0, 0, thumbWidth, thumbHeight);

        // Determine media type
        const mediaType = file.type || 'image/jpeg';

        // Convert to base64
        canvas.toBlob((blob) => {
          if (!blob) {
            reject(new Error('Failed to create blob from canvas'));
            return;
          }

          const fullReader = new FileReader();
          fullReader.onloadend = () => {
            if (!fullReader.result || typeof fullReader.result !== 'string') {
              reject(new Error('Failed to read blob as data URL'));
              return;
            }

            const base64Full = fullReader.result.split(',')[1];
            const thumbnail = thumbCanvas.toDataURL(mediaType, quality);

            resolve({
              base64: base64Full,
              mediaType: mediaType,
              thumbnail: thumbnail
            });
          };
          fullReader.onerror = reject;
          fullReader.readAsDataURL(blob);
        }, mediaType, quality);
      };

      img.onerror = reject;
      img.src = /** @type {string} */ (e.target.result);
    };

    reader.onerror = reject;
    reader.readAsDataURL(file);
  });
}
//needs to optimize this
export async function getIcon(weather) {
  
  const ollama = new Ollama({ host: "http://localhost:11434" });
  
  const response = await ollama.chat({
    model: 'gemma3:1b',
    "options": {
    //"seed": 101,
    "temperature": 0,
  },
    messages: [
      { role: 'system', content: `You job is to match a weather condition to an icon name in the list provided.  
      <instructions>
        -You will be provided with a weather condition and time of day (day or night) and will return an icon name from the list below that represents the weather condition and time of day provided. 
        - Match only to the icon names from the 'icon list'. 
        - Do not include or add any other text.  
        - If you are unable to find a match, return the icon name for 'sad_face'.
        - Do not return any other text than icon names that match from the provided 'icon list'.
        - Think about it in step. First understand the weather condition and time of day and then choose an icon name from the list that is closest to weather condition. 
        - Use this response format: { "iconName": "icon name from list" }
      </instructions>


      <icon list>
      - clear_night
      - cloudy_night
      - partly_cloudy_night
      - clear_foggy
      - foggy
      - clear_sunny
      - mostly_sunny
      - partly_sunny
      - rain
      - snow
      - thunderstorms
      - windy
      - sad_face
      </icon list>

        `},
      { role: 'user', content: weather }
    ],
  })
 console.log('respose',response.message.content)
  let iconName;
  try {
    const content = response.message.content.trim();
    let parsedContent;
    
    // Try to parse as JSON
    parsedContent = JSON.parse(content);
    
    // If it's an object with iconName property, use that; otherwise treat it as the icon name directly
    iconName = typeof parsedContent === 'object' && parsedContent.iconName 
      ? parsedContent.iconName 
      : parsedContent;
    
    // Validate that iconName is a string
    if (typeof iconName !== 'string') {
      console.warn(`Invalid iconName type, falling back to sad_face:`, iconName);
      return 'sad_face.svg';
    }
    
    // Whitelist validation for expected icon names
    const validIcons = [
      "clear_night", "cloudy_night", "partly_cloudy_night", "clear_foggy", "foggy",
      "clear_sunny", "mostly_sunny", "partly_sunny", "rain",
      "snow", "thunderstorms", "windy", "sad_face"
    ];
    if (!validIcons.includes(iconName)) {
      console.warn(`iconName "${iconName}" not valid, falling back to sad_face`);
      iconName = 'sad_face';
    }
  } catch (error) {
    console.warn('getIcon fallback, unable to parse/validate JSON:', error);
    return 'sad_face.svg'; // Fallback to existing icon
  }
  console.log('weather: ',weather, 'iconName:', iconName)
  return iconName + '.svg'
  
}


export function toggleTheme() {
  console.log('toggling theme')
  const body = document.body;
  const currentTheme = body.getAttribute('data-theme');
  
  if (currentTheme === 'dark') {
    body.setAttribute('data-theme', 'light');
    localStorage.setItem('theme', 'light');
  } else {
    body.setAttribute('data-theme', 'dark');
    localStorage.setItem('theme', 'dark');
  }
}

export function initTheme() {
  const savedTheme = localStorage.getItem('theme');
  if (savedTheme) {
    document.body.setAttribute('data-theme', savedTheme);
  } else {
    // Set default theme if no theme is saved
    document.body.setAttribute('data-theme', 'light');
  }
}

  export function addCopyButtonToPre() {
    console.log('addCopyButtonToPre');
    const preElements = document.querySelectorAll('pre');
    preElements.forEach(pre => {
      if (!pre.querySelector('.copy-button')) {
        console.log('adding copy button')
        const copyButton = document.createElement('button');
        copyButton.textContent = 'Copy';
        copyButton.className = 'copy-button';
        copyButton.addEventListener('click', () => {
          const code = pre.querySelector('code');
          if (code) {
            navigator.clipboard.writeText(code.textContent)
              .then(() => {
                copyButton.textContent = 'Copied!';
                setTimeout(() => {
                  copyButton.textContent = 'Copy';
                }, 2000);
              })
              .catch(err => {
                console.error('Failed to copy text: ', err);
              });
          }
        });
        
        pre.style.position = 'relative';
        pre.appendChild(copyButton);
      }
    });
  }

  export async function weatherReport(lat,lon) {
    try {

        const theWeather = document.querySelector("#weather .weather-report");
        const theWeatherDetails = document.querySelector("#weather .weather-details");
        const theIcon = document.querySelector("#weather .weather-icon");
        
        if (!theWeather) {
          console.error('Weather element not found in DOM');
          return;
        }
        
        theWeather.textContent = 'Loading...';

        console.log('Fetching weather for:', lat, lon);
        const report = await tauriFetch(
          `https://api.weather.gov/points/${lat},${lon}`,
          {
            method: 'GET',
            responseType: ResponseType.JSON,
            headers: {
              'User-Agent': 'Olly Weather App/1.0'
            }
          }
        );
        console.log('Weather report response:', report);
        console.log('Report data type:', typeof report.data);
        console.log('Report data:', report.data);
        console.log('Report status:', report.status);

        // The data might already be parsed JSON
        const locationGrid = typeof report.data === 'string' ? JSON.parse(report.data) : report.data;
        console.log('Location grid:', locationGrid);

        if (!locationGrid || !locationGrid.properties || !locationGrid.properties.forecast) {
          console.error('Invalid weather API response structure. LocationGrid:', locationGrid);
          throw new Error('Invalid weather API response structure');
        }

        const forecastURL = locationGrid.properties.forecast
        console.log('Forecast URL:', forecastURL);

        const forecastReturn = await tauriFetch(forecastURL, {
          method: 'GET',
          responseType: ResponseType.JSON,
          headers: {
            'User-Agent': 'Olly Weather App/1.0'
          }
        });
        console.log('Forecast response:', forecastReturn);

        const theForecast = typeof forecastReturn.data === 'string' ? JSON.parse(forecastReturn.data) : forecastReturn.data;
        console.log('Forecast data:', theForecast);

        const forecastDetails = theForecast.properties.periods[0]

        let iconWeather = `${forecastDetails.shortForecast}  ${forecastDetails.isDaytime ? 'Day' : 'Night'}`

        const icon = await getIcon(iconWeather)

        theIcon.style.mask = `url('/weather-icons/${icon}')`;
        theWeather.textContent = `${forecastDetails.name}: ${forecastDetails.temperature}°F`;
        // theWeatherDetails.textContent = `${forecastDetails.shortForecast}`;
    }
    catch (error) {
      console.error('Error fetching weather data:', error);
      const theWeather = document.querySelector("#weather .weather-report");
      if (theWeather) {
        theWeather.textContent = 'Weather unavailable'
      }
    }

    //return ;
  }
  export async function getCoordinates(city) {
  try {
    console.log('Fetching coordinates for city:', city);
    const location = await tauriFetch(
      `https://nominatim.openstreetmap.org/search?q=${city}&format=json`,
      {
        method: 'GET',
        responseType: ResponseType.JSON,
        headers: {
          'User-Agent': 'Olly Weather App/1.0'
        }
      }
    );
    console.log('Coordinates response:', location);
    console.log('Coordinates status:', location.status);
    console.log('Coordinates data type:', typeof location.data);

    const output = typeof location.data === 'string' ? JSON.parse(location.data) : location.data;
    console.log('Parsed location data:', output);

    if (output && output.length > 0) {
      const lat = output[0].lat;
      const lon = output[0].lon;
      console.log('Found coordinates:', lat, lon);
      weatherReport(lat,lon)
    } else {
      console.error('No location data found for city:', city);
      console.error('Full response:', location);
    }
  } catch (error) {
    console.error('Error fetching coordinates:', error);
    console.error('Error details:', error.message, error.stack);
  }
 //console.log('WeatherCity: ',lat,lon)
}
export function closeSettings() {
  const settingsDiv = document.querySelector('#settings');
  if (settingsDiv) {
    settingsDiv.classList.add('fadeOut');
    settingsDiv.addEventListener('animationend', () => {
      if (settingsDiv.classList.contains('fadeOut')) {
        settingsDiv.style.display = 'none';
        settingsDiv.classList.remove('fadeOut');
      }
    });
  }
}
export function openSettings() {
  const settingsDiv = document.querySelector('#settings');
  if (settingsDiv) {
    settingsDiv.style.display = 'flex';
    settingsDiv.classList.add('fadeIn');
  }
}

export function formatDate(dateString) {
  const date = new Date(dateString);
  const options = { 
    year: 'numeric', 
    month: 'short', 
    day: 'numeric', 
    hour: '2-digit', 
    minute: '2-digit', 
    //timeZoneName: 'short' 
  };
  return date.toLocaleString('en-US', options);
}

export function formatRelativeTime(dateString) {
  if (!dateString || dateString === "Unknown" || dateString.includes("External API")) {
    return dateString;
  }

  try {
    let date;
    
    // Try to parse as formatted date: "May 03, 2025 - 09:45 AM -04:00"
    const formattedDateMatch = dateString.match(/([A-Z][a-z]{2}) (\d{2}), (\d{4}) - (\d{2}):(\d{2}) ([AP]M) ([+-]\d{2}:\d{2}|[A-Z]{3,4})/);
    
    if (formattedDateMatch) {
      // Parse the matched components
      const [, month, day, year, hour, minute, ampm] = formattedDateMatch;
      
      // Convert to 24-hour format
      let hour24 = parseInt(hour);
      if (ampm === 'PM' && hour24 !== 12) hour24 += 12;
      if (ampm === 'AM' && hour24 === 12) hour24 = 0;
      
      // Create date string in a format JavaScript can parse
      const monthNames = ['Jan', 'Feb', 'Mar', 'Apr', 'May', 'Jun', 'Jul', 'Aug', 'Sep', 'Oct', 'Nov', 'Dec'];
      const monthIndex = monthNames.indexOf(month);
      
      if (monthIndex === -1) {
        return dateString;
      }
      
      // Create date object
      date = new Date(parseInt(year), monthIndex, parseInt(day), hour24, parseInt(minute));
    } else {
      // Try parsing as ISO date (RFC3339): "2025-11-07T20:34:17.377539404-05:00"
      date = new Date(dateString);
    }
    
    // Check if date is valid
    if (isNaN(date.getTime())) {
      return dateString;
    }
    
    const now = new Date();
    const diffMs = now.getTime() - date.getTime();
    const diffSecs = Math.floor(diffMs / 1000);
    const diffMins = Math.floor(diffSecs / 60);
    const diffHours = Math.floor(diffMins / 60);
    const diffDays = Math.floor(diffHours / 24);
    const diffWeeks = Math.floor(diffDays / 7);
    const diffMonths = Math.floor(diffDays / 30);
    const diffYears = Math.floor(diffDays / 365);

    if (diffSecs < 60) return "just now";
    if (diffMins < 60) return `${diffMins} minute${diffMins !== 1 ? 's' : ''} ago`;
    if (diffHours < 24) return `${diffHours} hour${diffHours !== 1 ? 's' : ''} ago`;
    if (diffDays < 7) return `${diffDays} day${diffDays !== 1 ? 's' : ''} ago`;
    if (diffWeeks < 4) return `${diffWeeks} week${diffWeeks !== 1 ? 's' : ''} ago`;
    if (diffMonths < 12) return `${diffMonths} month${diffMonths !== 1 ? 's' : ''} ago`;
    return `${diffYears} year${diffYears !== 1 ? 's' : ''} ago`;
  } catch (error) {
    return dateString;
  }
}