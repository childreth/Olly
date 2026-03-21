/** @type {import('./$types').PageServerLoad} */
import { Ollama } from "ollama/browser";

//pulls current models available in Ollama

export async function load() {
    const ollama = new Ollama({ host: "http://localhost:11434" });

    let modelNames = [];
    try {
        let models = await ollama.list();
        let theModels = models.models;


        modelNames = theModels.map(modelName => {
            return modelName.name;
        })
    } catch (e) {
        console.warn("Failed to connect to ollama locally during build/load", e);
    }
    //manually add names here

    modelNames.unshift("Fal - Flux");
   

    return {modelNames};
};