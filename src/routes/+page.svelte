<script>
  import { invoke } from "@tauri-apps/api/tauri";
  import { listen } from "@tauri-apps/api/event";
  import { Ollama } from "ollama/browser";
  import { onMount } from "svelte";
  import { marked } from "marked";
  import * as Utils from "$lib/utils.js";
  import SendButton from "$lib/components/sendButton.svelte";
  import Button from "$lib/components/button.svelte";
  import Select from "$lib/components/select.svelte";
  import ColorPicker from "$lib/components/colorPicker.svelte";
  import Toggle from "$lib/components/darkModeToggle.svelte";
  import SettingsModal from "$lib/components/settingsModal.svelte";
  import Toast from "$lib/components/toast.svelte";
  import SearchableSelect from "$lib/components/searchableSelect.svelte";
  import { appWindow } from "@tauri-apps/api/window";

  import { open } from "@tauri-apps/api/dialog";
  import { confirm } from "@tauri-apps/api/dialog";

  import { fetch, ResponseType } from "@tauri-apps/api/http";
  // Use this to get the environment variable


  //basic API call for testing tools
  const API_URL = "https://rickandmortyapi.com/api/episode";
  let selectedModel = "smollm2:1.7b";
  let selectedModelOption = null;
  let activeModel = "";
  let result = "";
  let theImage= [];
  let theThumbnail = "";
  let countConvo = 0;
  let userMsg = "tell me a dad joke";
  let lastChatResponse = "";
  let streamedGreeting = "";
  $: responseMarked = marked.parse(streamedGreeting);
  let chatConvo = [];
  let tokenSpeed = 0;
  let tokenCount = 0;
  const city = "Westford,MA";
  let name = "Craig";
  let loadModelNames = [];
  let allModels = [];
  let isStreaming = false;
  let abortController = new AbortController();
  const ollama = new Ollama({ host: "http://localhost:11434" });

  let darkMode = false;
  let themeColor = "#000099"; // default color
  let showSettings = false;
  
  // Toast notification state
  let toastVisible = false;
  let toastMessage = "";
  let toastType = "info";

//very basic system prompt to test speed vs terimal interface
  const systemMsg = `You are a somewhat helpful assistant and like really responsing with emojies. You job will be to help write better content for the user.`;

  
  onMount(async () => {
    // Listen for API key migration events
    const unsubscribe = await listen("api-keys-migrated", (event) => {
      toastMessage = event.payload;
      toastType = "success";
      toastVisible = true;
    });
    
    const savedColor = localStorage.getItem('themeColor');
    if (savedColor) {
      themeColor = savedColor;
      const hexToHSL = (hex) => {
        hex = hex.replace(/^#/, "");
        let r = parseInt(hex.slice(0, 2), 16) / 255;
        let g = parseInt(hex.slice(2, 4), 16) / 255;
        let b = parseInt(hex.slice(4, 6), 16) / 255;
        let cmin = Math.min(r, g, b),
          cmax = Math.max(r, g, b),
          delta = cmax - cmin,
          h = 0;
        if (delta === 0) h = 0;
        else if (cmax === r) h = ((g - b) / delta) % 6;
        else if (cmax === g) h = (b - r) / delta + 2;
        else h = (r - g) / delta + 4;
        h = Math.round(h * 60);
        if (h < 0) h += 360;
        return h;
      };
      document.documentElement.style.setProperty("--hue", hexToHSL(savedColor).toString());
    }

    const sendBtn = document.querySelector("#sendBtn");
    const imagePreview = document.querySelector("#thumbnails");
    const prompt = document.querySelector("#prompt");
    const apiKey = await invoke('get_env', { name: 'CLAUDE_API_KEY' });
    
    

    Utils.getCoordinates(city);
    loadModels();

    const fileInput = document.querySelector("#file");

    prompt.addEventListener("keydown", (e) => {
      if (e.key === "Enter") {
        callOllama()
      }
    });

    fileInput.addEventListener("change", (e) => {
      const file = fileInput.files[0];
      const reader = new FileReader();

      reader.addEventListener("load", () => {
        let uploadedImg = reader.result.split(",")[1];
        theThumbnail = reader.result;
        theImage.push(uploadedImg);
        // for thumbnail
        let thumbnail = document.createElement("img");
        thumbnail.src = reader.result;
        imagePreview?.appendChild(thumbnail);
        //document.querySelector("#thumbs").src = reader.result;

        //let theImageURL = URL.createObjectURL(fileInput.files[0]);
        //theImage = fileInput.files[0];1
        console.log("reader: ", theImage);
      });
      reader.readAsDataURL(file);
    });
    document
      .getElementById("titlebar-minimize")
      .addEventListener("click", () => appWindow.minimize());
    document
      .getElementById("titlebar-maximize")
      .addEventListener("click", () => appWindow.toggleMaximize());
    // document
    //   .getElementById("titlebar-close")
    //   .addEventListener("click", () => appWindow.close());
    
    // Claude streaming event listeners
    appWindow.listen('claude-stream', (event) => {
      const content = event.payload;
      streamedGreeting += content;
      lastChatResponse += content;
    });
    
    appWindow.listen('claude-stream-done', (event) => {
      console.log("Claude streaming completed");
      isStreaming = false;
      Utils.addCopyButtonToPre();
    });

    // Perplexity streaming event listeners
    appWindow.listen('perplexity-stream', (event) => {
      const content = event.payload;
      streamedGreeting += content;
      lastChatResponse += content;
    });
    
    appWindow.listen('perplexity-stream-done', (event) => {
      console.log("Perplexity streaming completed");
      isStreaming = false;
      Utils.addCopyButtonToPre();
    });
    
    //callOllama()
  });

  async function rickAndMorty() {
    const response = await fetch(API_URL, {
      method: "GET",
      timeout: 30,
      responseType: ResponseType.JSON,
    });
    console.log("response", response);
  }
  // Open a selection dialog for image files
  async function showDialog() {
    try {
      const selected = await open({
        multiple: true,
        filters: [
          {
            name: "Image",
            extensions: ["png", "jpeg"],
          },
        ],
      });

      if (Array.isArray(selected)) {
        console.log("User selected multiple files:", selected);
        // Handle multiple file selection
      } else if (selected === null) {
        console.log("User cancelled the selection");
        // Handle cancellation
      } else {
        console.log("User selected a single file:", selected);
        // Handle single file selection
      }
    } catch (error) {
      console.error("Error opening dialog:", error);
    }
  }
  //basic confirm dialog
  async function confirmDialog() {
    try {
      confirm("Are you sure?", "Tauri");
    } catch (error) {
      console.error("Error opening dialog:", error);
    }
  }

  async function askClaude(userMsg) {
    try {
      isStreaming = true;
      lastChatResponse = "";
      await invoke('stream_claude', {
        prompt: userMsg
      });
    } catch (error) {
      console.error(error);
      isStreaming = false;
      
      // Show error to user
      if (error.includes("API key not found")) {
        toastMessage = "Claude API key not found. Please add it in Settings → API Settings.";
        toastType = "error";
        toastVisible = true;
      } else {
        toastMessage = `Claude error: ${error}`;
        toastType = "error";
        toastVisible = true;
      }
    }
  }

  async function askPerplexity(userMsg) {
    try {
      isStreaming = true;
      lastChatResponse = "";
      await invoke('stream_perplexity', {
        model: selectedModel,
        prompt: userMsg
      });
    } catch (error) {
      console.error(error);
      isStreaming = false;
      
      // Show error to user
      if (error.includes("API key not found")) {
        toastMessage = "Perplexity API key not found. Please add it in Settings → API Settings.";
        toastType = "error";
        toastVisible = true;
      } else {
        toastMessage = `Perplexity error: ${error}`;
        toastType = "error";
        toastVisible = true;
      }
    }
  }




  async function loadModels() {
    try {
      // Get all models from backend
      const backendModels = await invoke("get_all_models");
      console.log('Backend models:', backendModels);
      
      // Get Ollama models if available
      let ollamaModels = [];
      try {
        const ollama = new Ollama({ host: "http://localhost:11434" });
        let models = await ollama.list();
        ollamaModels = models.models.map((modelName) => ({
          id: modelName.name,
          name: modelName.name,
          description: `${modelName.details.parameter_size} - ${Utils.formatDate(modelName.modified_at)}`,
          provider: "ollama",
          details: {
            modified_at: modelName.modified_at,
            parameter_size: modelName.details.parameter_size,
            quantization_level: modelName.details.quantization_level
          }
        }));
        
        // Keep old format for settings display
        loadModelNames = models.models.map((modelName) => [
          modelName.name,  
          Utils.formatDate(modelName.modified_at),
          modelName.details.parameter_size,
          modelName.details.quantization_level
        ]);
      } catch (error) {
        console.warn("Ollama not available:", error);
        loadModelNames = [];
      }
      
      // Add external API models to old format for settings
      loadModelNames.unshift(["Fal - Flux","Not local - External API","N/A","N/A"],["Claude 3.5 Sonnet","Not local - External API","N/A","N/A"]);
      loadModelNames.push(...backendModels.filter(m => m.provider === "perplexity").map(m => [m.name, "Not local - External API", "N/A", "N/A"]));
      
      // Combine all models for the searchable select
      allModels = [...backendModels, ...ollamaModels];
      console.log('All models combined:', allModels);
      
    } catch (error) {
      console.error("Failed to load models:", error);
      // Fallback models
      loadModelNames = [["Fal - Flux","Not local - External API","N/A","N/A"],["Claude 3.5 Sonnet","Not local - External API","N/A","N/A"]];
      allModels = [
        { id: "claude-3.5-sonnet", name: "Claude 3.5 Sonnet", description: "Most capable Claude model", provider: "claude" },
        { id: "fal-flux", name: "Fal - Flux", description: "Image generation model", provider: "fal" }
      ];
    }
  }

  async function deleteModel(model) {
    const ollama = new Ollama({ host: "http://localhost:11434" });
    let processing = ollama.ps()
    let models = await ollama.delete({ model: model });
    console.log("deleteModel:", processing);
    loadModels()
    
  }

  async function callOllama() {
    
    userMsg = document.querySelector("#prompt").textContent || "";

    //add user message to the top of the chat
    streamedGreeting += `<h2 class="userMsg"> <span>${userMsg}</span>` + 
      `${theThumbnail ? `<img src="${theThumbnail}" alt="User uploaded image">` : ""}
      </h2>`;
    
    
    streamedGreeting += `<p><small><strong>${selectedModel}</strong></small></p>`
    document.querySelector("#prompt").textContent = "";
    document.querySelector("#thumbnails").innerHTML = "";

    

    //add user message to the thread
    if (countConvo == 0) {
      chatConvo[countConvo++] = { role: "system", content: systemMsg };
      chatConvo[countConvo++] = {
        role: "user",
        content: userMsg,
        images: theImage,
      };
      //clear the image array
      theImage = [];
      //chatConvo[countConvo++] = { role: "user", images: [theImage] };
    } else {
      chatConvo[countConvo++] = {
        role: "assistant",
        content: lastChatResponse,
        images: theImage,
      };
      chatConvo[countConvo++] = {
        role: "user",
        content: userMsg,
        images: theImage,
      };
    }

    // Determine provider from selected model
    const provider = selectedModelOption ? selectedModelOption.provider : 'ollama';
    
    if (selectedModel === "fal-flux" || selectedModel === "Fal - Flux") {
      falImage();
    } else if (provider === "claude") {
      askClaude(userMsg);
    } else if (provider === "perplexity") {
      askPerplexity(userMsg);
    } else {
      console.log("chatConvo:", chatConvo);
      isStreaming = true;
      abortController = new AbortController();
      lastChatResponse = "";

      try {
        const response = await ollama.chat({
          model: selectedModel,
          messages: chatConvo,
          stream: true,
          options: {
            temperature: 0.9,
          },
          signal: abortController.signal,
        });

       

        for await (const part of response) {
          if (abortController.signal.aborted) {
            ollama.abort();
            break;
          }
          streamedGreeting += part.message.content;
          lastChatResponse += part.message.content;
          //looks for end of stream
          if (part.eval_count) {
            tokenCount = Number(part.eval_count);
            tokenSpeed = Number(
              (part.eval_count / part.eval_duration) * Math.pow(10, 9)
            ).toFixed(2);
          }
        }
        console.log("streamGreet:", streamedGreeting);
      } catch (error) {
        if (error.name === "AbortError") {
          console.log("Stream was aborted");
        } else {
          console.error("Error during streaming:", error);
        }
      } finally {
        isStreaming = false;
        Utils.addCopyButtonToPre();
        //streamedGreeting += ``;
      }
    }

    // sendBtn.disabled = false;
    // sendBtn.textContent = "Send";
  }

  function autoScroll() {
    const chatContainer = document.querySelector("#chat-container");
    if (chatContainer) {
      chatContainer.scrollTop = chatContainer.scrollHeight;
    }
  }

  $: if (streamedGreeting) {
    // Use setTimeout to ensure the DOM has updated before scrolling
    setTimeout(autoScroll, 0);
  }

  function changeModel() {
    //reset the chat for new conversation+model
    console.log("model reset");
    countConvo = 0;
    chatConvo = [];
    lastChatResponse = "";
    theImage = [];
    tokenCount = 0;
    tokenSpeed = 0;
    document.querySelector("#thumbnails").innerHTML = "";
    // ollama.stop();
  }

  function handleModelChange(event) {
    const { value, option } = event.detail;
    selectedModel = value;
    selectedModelOption = option;
    changeModel(); // Reset conversation when changing model
  }
  function stopStreaming() {
    if (isStreaming) {
      abortController.abort();
      isStreaming = false;
      sendBtn.disabled = false;
      sendBtn.textContent = "Send";
    }
  }

  function handleColorChange(event) {
    const { color } = event.detail;
    localStorage.setItem('themeColor', color);
  }
</script>

<div id="settings">
  <div class="settings-content">
    <header>
      <h3>Settings</h3><button class="basic" on:click={Utils.closeSettings}>Close</button>
    </header>

    <section>
      <h4>General</h4>
      <p style='display:flex;flex-direction:row;align-items:flex-end;gap:2.5rem;'>

        <Select id='typeface' small={true} />
        <Toggle  id="darkModeToggle"  />
        <ColorPicker color={themeColor} on:colorChange={handleColorChange} />
        <Button label="API Settings" on:click={() => showSettings = true} />
      </p>
      <h4>Manage models</h4>
    <ul>
      <li class='thead'><span>Name</span> <span class='date'>Last updated</span> <span>Parameter</span><span>Quantization</span><span class='actions'>&nbsp;</span></li>
      {#each loadModelNames as model}
        <li>
          <span>{model[0]}</span>
          <span class='date'>{model[1]}</span>
          <span>{model[2]}</span>
          <span>{model[3]}</span>
          <span>
            <button class='basic' on:click={deleteModel(`${model[0]}`)}>Delete</button></span></li>
      {/each}
    </ul>
  </section>
  </div>
  
</div>

<header id="title">
  <div id="weather">
    <span class="weather-icon"></span>
    <div><span class="weather-report"></span></div>
    <div style="margin:0 1rem;color:var(--secondary)"> | </div><!-- <div class="weather-details"></div> -->
    <a class='basic' on:click={Utils.openSettings}>Settings</a>
  </div>
  <!-- <button on:click={confirmDialog}>Show Dialog</button> -->
  <h1>Olly</h1>
  <!-- <button class="basic" on:click={Utils.toggleTheme}>Test it</button> -->
  <div class="input-vertical">
    <label for="model-select" class="visualhide">Choose a model:</label>
    <SearchableSelect 
      options={allModels}
      bind:value={selectedModel}
      on:change={handleModelChange}
      placeholder="Search models..."
    />
  </div>
</header>
<main>
 
  <div id="chat-container">
    <section id="" class="response" aria-live="polite" role="log">
      {@html responseMarked}
    </section>
    
  </div>
 

  <div id="userinput" class="halftone">
    <div class="combotext">
      <div id="imgInput">
        <label for="file" class="custom-file-upload"
          ><span class="visualhide">Upload an image</span></label
        >
        <input type="file" id="file" />
      </div>
      <div id="thumbnails"></div>
      <div class="textarea-container">
        <label id="promptLabel" for="prompt" class="visualhide"
          >Add your prompt:</label
        >

        <div
          id="prompt"
          role="textbox"
          class="fakeTextarea"
          contenteditable="true"
          aria-labelledby="promptLabel"
          aria-describedby="highlights"
          aria-details="highlights"
          spellcheck="false"
          placeholder="How can I help?"
        ></div>
      </div>

      <div id="buttonContainer">
        <SendButton
          label={isStreaming ? "Stop" : "Start"}
          on:click={isStreaming ? stopStreaming : callOllama}
          elID={isStreaming ? "stopBtn" : "sendBtn"}
        />
      </div>
    </div>
    <!-- <audio id="speech" controls style="position: fixed; bottom: 0; left: 0; width: 100%;" /> -->
    <p class="modelInfo">
       &nbsp; <strong>{selectedModel}</strong>:
      <span class="highlightText">{tokenSpeed} tokens/sec</span>
      &mdash; <span class="highlightText">{tokenCount} total tokens</span> 
      
    </p>
  </div>
</main>

<SettingsModal bind:isOpen={showSettings} />
<Toast bind:visible={toastVisible} message={toastMessage} type={toastType} />

<style lang="">
  @import "./styles.css";
  @import "./darkmode.css";
  @import "./animation.css";
</style>
