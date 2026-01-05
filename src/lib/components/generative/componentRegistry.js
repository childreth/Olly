import WeatherCard from './WeatherCard.svelte';

/**
 * Maps tool names to their associated Svelte components
 * When a tool returns data with a _component field, this registry
 * is used to find and render the appropriate component
 * @type {Record<string, any>}
 */
export const componentRegistry = {
  'WeatherCard': WeatherCard,
  // Future components can be added here
  // 'CalendarList': CalendarList,
  // 'EventCard': EventCard,
};

/**
 * Get a component from the registry by name
 * @param {string} componentName - Name of the component to retrieve
 * @returns {any} - The Svelte component or null if not found
 */
export function getComponent(componentName) {
  return componentRegistry[componentName] || null;
}

/**
 * Check if a component exists in the registry
 * @param {string} componentName - Name of the component to check
 * @returns {boolean} - True if component exists
 */
export function hasComponent(componentName) {
  return componentName in componentRegistry;
}
