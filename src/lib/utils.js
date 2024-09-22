import { Ollama } from "ollama/browser";
export { Ollama }
//needs to optimize this
export async function getIcon(weather) {
  
  const ollama = new Ollama({ host: "http://localhost:11434" });

  const response = await ollama.chat({
    model: 'qwen2.5:1.5b',
    messages: [
      { role: 'system', content: `You job is to match a weather condition to an icon name in the list below.  You will be provided with a weather condition and will return an icon name from the list below that represents the weather condition provided.  Return only the icon names from the list. Do not includ or add any other text.  If you are unable to find a match, return the icon name for 'sad_face' Read through the weather condition and list twice before sending a response.
      <icon list names>
      - clear_night
      - partly_cloudy_night
      - clear_foggy
      - foggy
      - sunny
      - partly_cloudy_sunny
      - rain
      - snow
      - thunderstorms
      - windy
      </icon list names>

        `},
      { role: 'user', content: weather }
    ],
  })
  let iconName = (response.message.content).replace(/[\r\n\s]/g,'')+'.svg'
  return iconName
  
}


  export function addCopyButtonToPre() {
    console.log('adding copy button')
    const preElements = document.querySelectorAll('pre');
    preElements.forEach(pre => {
      if (!pre.querySelector('.copy-button')) {
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

        const theWeather = document.querySelector("#weather");
        theWeather.textContent = 'Loading...';

        const report = await fetch(
          `https://api.weather.gov/points/${lat},${lon}`
        );
        const locationGrid = await report.json();
        const forecastURL = locationGrid.properties.forecast
        
        const forecastReturn = await fetch(forecastURL);
        const theForecast = await forecastReturn.json();
        const forecastDetails = theForecast.properties.periods[0]
        

        const icon = await getIcon(forecastDetails.shortForecast)
        
        theWeather.style.backgroundImage = `url('/src/lib/images/weather/${icon}')`;
        theWeather.textContent = `${forecastDetails.name}: ${forecastDetails.temperature}°F - ${forecastDetails.shortForecast}`
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