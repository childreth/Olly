<script>
  /** @type {{ location?: string, forecast?: Array<{ name: string, temperature: number, temperatureUnit: string, shortForecast: string, windSpeed?: string, precipitationProbability?: number }> }} */
  export let data = {};
  
  const { location = 'Unknown', forecast = [] } = data;
  
  /**
   * Get weather emoji based on forecast description
   * @param {string} shortForecast - Weather forecast text
   * @returns {string} - Emoji representation
   */
  function getWeatherEmoji(shortForecast) {
    const forecast_lower = shortForecast?.toLowerCase() || '';
    
    if (forecast_lower.includes('sunny') || forecast_lower.includes('clear')) return '☀️';
    if (forecast_lower.includes('cloudy') || forecast_lower.includes('overcast')) return '☁️';
    if (forecast_lower.includes('rain') || forecast_lower.includes('shower')) return '🌧️';
    if (forecast_lower.includes('snow')) return '❄️';
    if (forecast_lower.includes('thunder') || forecast_lower.includes('storm')) return '⛈️';
    if (forecast_lower.includes('fog') || forecast_lower.includes('mist')) return '🌫️';
    if (forecast_lower.includes('wind')) return '💨';
    if (forecast_lower.includes('partly')) return '⛅';
    
    return '🌤️';
  }
  
  /**
   * Format temperature to integer
   * @param {number} temp - Temperature value
   * @returns {number} - Rounded temperature
   */
  function formatTemp(temp) {
    return Math.round(temp);
  }
</script>

<div class="weather-container">
  <div class="weather-header">
    <h3 class="location">📍 {location}</h3>
  </div>
  
  <div class="forecast-grid">
    {#each forecast as period, index (index)}
      <div class="weather-card">
        <div class="card-header">
          <div class="day-name">{period.name}</div>
          <div class="emoji">{getWeatherEmoji(period.shortForecast)}</div>
        </div>
        
        <div class="card-body">
          <div class="temperature-section">
            <div class="current-temp">
              <span class="temp-value">{formatTemp(period.temperature)}</span>
              <span class="temp-unit">°{period.temperatureUnit}</span>
            </div>
          </div>
          
          <div class="conditions">
            <p class="forecast-text">{period.shortForecast}</p>
          </div>
          
          <div class="details">
            {#if period.windSpeed}
              <div class="detail-item">
                <span class="detail-label">Wind:</span>
                <span class="detail-value">{period.windSpeed}</span>
              </div>
            {/if}
            
            {#if period.precipitationProbability}
              <div class="detail-item">
                <span class="detail-label">Precip:</span>
                <span class="detail-value">{period.precipitationProbability}%</span>
              </div>
            {/if}
          </div>
        </div>
      </div>
    {/each}
  </div>
</div>

<style scoped>
  .weather-container {
    width: 100%;
    padding: 1rem;
    background: linear-gradient(135deg, var(--surface-2) 0%, var(--surface-3) 100%);
    border-radius: var(--borderRadiusSM);
    margin: 1rem 0;
  }
  
  .weather-header {
    margin-bottom: 1.5rem;
  }
  
  .location {
    margin: 0;
    font-size: var(--fontSizeLarge);
    color: var(--primary);
    font-weight: 600;
  }
  
  .forecast-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
    gap: 1rem;
  }
  
  .weather-card {
    background: var(--surface-1);
    border-radius: var(--borderRadiusXS);
    padding: 1rem;
    border: 1px solid var(--surface-3);
    transition: all 0.2s ease;
    box-shadow: 0 2px 4px rgba(0, 0, 0, 0.05);
  }
  

  
  .card-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 1rem;
    padding-bottom: 0.75rem;
    border-bottom: 1px solid var(--surface-3);
  }
  
  .day-name {
    font-weight: 600;
    color: var(--primary);
    font-size: var(--fontSizeMedium);
  }
  
  .emoji {
    font-size: 1.75rem;
  }
  
  .card-body {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }
  
  .temperature-section {
    display: flex;
    align-items: baseline;
    gap: 0.5rem;
  }
  
  .current-temp {
    display: flex;
    align-items: baseline;
    gap: 0.25rem;
  }
  
  .temp-value {
    font-size: var(--fontSizeXLarge);
    font-weight: 700;
    color: var(--primary);
  }
  
  .temp-unit {
    font-size: var(--fontSizeSmall);
    color: var(--secondary);
    font-weight: 500;
  }
  
  .conditions {
    margin: 0.5rem 0;
  }
  
  .forecast-text {
    margin: 0;
    font-size: var(--fontSizeSmall);
    color: var(--secondary);
    line-height: 1.4;
  }
  
  .details {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    margin-top: 0.5rem;
    padding-top: 0.75rem;
    border-top: 1px solid var(--surface-3);
  }
  
  .detail-item {
    display: flex;
    justify-content: space-between;
    font-size: var(--fontSizeSmall);
  }
  
  .detail-label {
    color: var(--secondary);
    font-weight: 500;
  }
  
  .detail-value {
    color: var(--primary);
    font-weight: 600;
  }
  
  @media (max-width: 768px) {
    .forecast-grid {
      grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
    }
    
    .weather-container {
      padding: 0.75rem;
    }
  }
</style>
