import { Ollama } from "ollama/browser";
export { Ollama }
//needs to optimize this
export async function getIcon(weather) {
  
  const ollama = new Ollama({ host: "http://localhost:11434" });
  
  const response = await ollama.chat({
    model: 'smollm2:360m',
    "options": {
    //"seed": 101,
    "temperature": 0
  },
    messages: [
      { role: 'system', content: `You job is to match a weather condition to an icon name in the list provided.  
      <instructions>
        -You will be provided with a weather condition and time of day (day or night) and will return an icon name from the list below that represents the weather condition and time of day provided. 
        - Match only to the icon names from the 'icon list'. 
        - Do not include or add any other text.  
        - If you are unable to find a match, return the icon name for 'sad_face'.
        - Do not return any other text than icon names that match from the provided 'icon list'.
        - Return in json format 
      </instructions>

      <response format>
        { "iconName": "icon name from list" }
      }
      </response format>

      <icon list>
      - clear_night
      - cloudy_night
      - clear_foggy
      - foggy
      - clear_sunny
      - mostly_sunny
      - partly_cloudy_sunny
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
 // let iconName = (response.message.content).replace(/[\r\n\s]/g,'')+'.svg'
  let responseObject = JSON.parse(response.message.content)
  let iconName = responseObject.iconName + '.svg'
  console.log('weather: ',weather, responseObject.iconName)
  return iconName
  
}

export function toggleTheme() {
  console.log('toggling theme')
  const body = document.body;
  const currentTheme = body.getAttribute('data-theme');
  
  if (currentTheme === 'dark') {
    body.setAttribute('data-theme', 'light');
    // localStorage.setItem('theme', 'light');
  } else {
    body.setAttribute('data-theme', 'dark');
    // localStorage.setItem('theme', 'dark');
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
        
        theIcon.style.mask = `url('/src/lib/images/weather/${icon}')`;
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




