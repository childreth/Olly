/** @type {import('./$types').PageServerLoad} */
import ollama from 'ollama'

//pulls current models available in Ollama

export async function load() {
    let models = await ollama.list();
    let theModels = models.models;
   
    
    const modelNames = theModels.map(modelName => {
        return modelName.name;
    })
    //manually add names here

    modelNames.unshift("Fal - Flux","Canary Chrome");
   

    return {modelNames};
};

