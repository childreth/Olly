/** @type {import('./$types').PageServerLoad} */
import { Ollama } from "ollama/browser";

//pulls current models available in Ollama

export async function load() {
    const ollama = new Ollama({ host: "http://localhost:11434" });

    let models = await ollama.list();
    let theModels = models.models;
   
    
    const modelNames = theModels.map(modelName => {
        return modelName.name;
    })
    //manually add names here

    modelNames.unshift("Fal - Flux","Canary Chrome");
   

    return {modelNames};
};

