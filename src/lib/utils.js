import { Ollama } from "ollama/browser";
export { Ollama }
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
        theWeather.textContent = 'Loading...';

        const report = await fetch(
          `https://api.weather.gov/points/${lat},${lon}`
        );
        console.log('report: ',report)
        const locationGrid = await report.json();
        const forecastURL = locationGrid.properties.forecast
        
        const forecastReturn = await fetch(forecastURL);
        const theForecast = await forecastReturn.json();
        const forecastDetails = theForecast.properties.periods[0]
        
        let iconWeather = `${forecastDetails.shortForecast}  ${forecastDetails.isDaytime ? 'Day' : 'Night'}`

        const icon = await getIcon(iconWeather)
        
        theIcon.style.mask = `url('/weather-icons/${icon}')`;
        theWeather.textContent = `${forecastDetails.name}: ${forecastDetails.temperature}°F`;
        // theWeatherDetails.textContent = `${forecastDetails.shortForecast}`;
    }
    catch (error) {
      console.error('Error fetching weather data:', error);
      theWeather.textContent = 'Current weather unavailable.'
    }
    
    //return ;
  }
  export async function getCoordinates(city) {
  const location = await fetch(`https://nominatim.openstreetmap.org/search?q=${city}&format=json`,);
  const output = await location.json();
  const lat = output[0].lat;
  const lon = output[0].lon;
  weatherReport(lat,lon)
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
  if (!dateString) return dateString;

  try {
    // Parse date string - handle format: "May 03, 2025 - 09:45 AM -04:00"
    const dateMatch = dateString.match(/([A-Z][a-z]{2}) (\d{2}), (\d{4}) - (\d{2}):(\d{2}) ([AP]M) ([+-]\d{2}:\d{2}|[A-Z]{3,4})/);
    
    if (!dateMatch) {
      return dateString;
    }
    
    // Parse the matched components
    const [, month, day, year, hour, minute, ampm] = dateMatch;
    
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
    
    // Create date object (ignoring timezone for simplicity)
    const date = new Date(parseInt(year), monthIndex, parseInt(day), hour24, parseInt(minute));
    
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