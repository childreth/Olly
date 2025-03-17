<script>
  import { invoke } from "@tauri-apps/api/tauri";
  import { Ollama } from "ollama/browser";
  import { onMount } from "svelte";
  import { marked } from "marked";
  import * as Utils from "$lib/utils.js";
  import SendButton from "$lib/components/sendButton.svelte";
  import Button from "$lib/components/button.svelte";
  import Select from "$lib/components/select.svelte";
  import ColorPicker from "$lib/components/colorPicker.svelte";
  import Toggle from "$lib/components/darkModeToggle.svelte";
  import { appWindow } from "@tauri-apps/api/window";
  import { listen } from '@tauri-apps/api/event';

  import { open } from "@tauri-apps/api/dialog";
  import { confirm } from "@tauri-apps/api/dialog";

  import { fetch, ResponseType } from "@tauri-apps/api/http";
  // Use this to get the environment variable


  //basic API call for testing tools
  const API_URL = "https://rickandmortyapi.com/api/episode";
  let selectedModel = "smollm2:1.7b";
  let activeModel = "";
  let result = "";
  let theImage= [];
  let theThumbnail= [];
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
  let isStreaming = false;
  let abortController = new AbortController();
  const ollama = new Ollama({ host: "http://localhost:11434" });
  let perplexityApiKey = "";

  let claudeApiKey = '';
  let unlisten = null;
  let unlistenPerplexity = null;
  let unlistenPerplexityDone = null;
  let unlistenClaude = null;
  let unlistenClaudeDone = null;

  let darkMode = false;
  let themeColor = "#000099"; // default color

//very basic system prompt to test speed vs terimal interface
  const systemMsg = `You are a somewhat helpful assistant and like really responsing with emojies. You job will be to help write better content for the user. Return response with markdown formating. Choose appropriate markdown elements for the content asked for.`;

  // Configure marked with minimal options
  marked.setOptions({
    breaks: true,    // Convert \n to <br>
    gfm: true        // Use GitHub Flavored Markdown
  });

  onMount(async () => {
    const savedColor = localStorage.getItem('themeColor');
    if (savedColor) {
      themeColor = savedColor;
      document.documentElement.style.setProperty('--themeColor', themeColor);
      document.documentElement.style.setProperty('--themeColor-light', Utils.lightenColor(themeColor, 20));
      document.documentElement.style.setProperty('--themeColor-lighter', Utils.lightenColor(themeColor, 40));
    }

    const savedDarkMode = localStorage.getItem('darkMode');
    if (savedDarkMode === 'true') {
      darkMode = true;
      document.documentElement.classList.add('dark-mode');
    }

    const prompt = document.querySelector("#prompt");
    const apiKey = await invoke('get_env', { name: 'CLAUDE_API_KEY' });
    perplexityApiKey = await invoke('get_env', { name: 'PERPLEXITY_API_KEY' });
    claudeApiKey = await invoke('get_env', { name: 'CLAUDE_API_KEY' });

    // Listen for Perplexity streaming events
    unlistenPerplexity = await listen('perplexity-stream', (event) => {
      streamedGreeting += event.payload;
      Utils.addCopyButtonToPre();
    });

    // Listen for Perplexity streaming completion
    unlistenPerplexityDone = await listen('perplexity-stream-done', (event) => {
      console.log('Perplexity stream completed, full response:', event.payload);
      lastChatResponse = event.payload;
      isStreaming = false;
      Utils.addCopyButtonToPre();
    });

    // Listen for Claude streaming events
    unlistenClaude = await listen('claude-stream', (event) => {
      streamedGreeting += event.payload;
    });

    // Listen for Claude stream completion
    unlistenClaudeDone = await listen('claude-stream-done', (event) => {
      console.log('Claude streaming completed, full response:', event.payload);
    });

    Utils.getCoordinates(city);
    loadModels();

    const fileInput = document.querySelector("#file");

    if (prompt) {
      prompt.addEventListener('keydown', (e) => {
        if (e.key === 'Enter' && !e.shiftKey) {
          e.preventDefault();
          callOllama();
        }
      });
    }

    if (fileInput) {
      fileInput.addEventListener('change', () => {
        if (fileInput && fileInput.files && fileInput.files[0]) {
          const reader = new FileReader();
          reader.onload = (e) => {
            if (reader.result) {
              if (typeof reader.result === 'string') {
                theImage = reader.result.split(',');
                theThumbnail = reader.result;
                console.log('image uploaded')
              }
            }
          };
          reader.readAsDataURL(fileInput.files[0]);
        }
      });
    }

    document
      .getElementById("titlebar-minimize")
      .addEventListener("click", () => appWindow.minimize());
    document
      .getElementById("titlebar-maximize")
      .addEventListener("click", () => appWindow.toggleMaximize());
    // document
    //   .getElementById("titlebar-close")
    //   .addEventListener("click", () => appWindow.close());
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
      
      // Clear previous response
      streamedGreeting = "";
      
      // Add the user's message to the conversation history
      streamedGreeting += `<h2 class="userMsg"> <span>${userMsg}</span>` + 
      `${theThumbnail != "" ? `<img src="${theThumbnail}" alt="User uploaded image">` : ""}
      </h2>`;
      
      // Set up the messages array
      const messages = [];
      
      // Add user message
      messages.push({
        role: "user",
        content: userMsg
      });
      
      console.log("Sending request to Claude API with message:", userMsg);
      
      // Call the stream_claude Rust function
      await invoke('stream_claude', {
        prompt: userMsg
      });
      
      isStreaming = false;
    } catch (error) {
      streamedGreeting = `Error: ${error}`;
      isStreaming = false;
      console.error("Error calling Claude API:", error);
    }
  }

  async function askPerplexity(userMsg) {
    try {
      if (!perplexityApiKey) {
        streamedGreeting += "Error: PERPLEXITY_API_KEY environment variable is not set. Please add it to your .env file.";
        return;
      }

      // Format messages for Perplexity API
      let messages = chatConvo.map(msg => {
        return {
          role: msg.role,
          content: msg.content
        };
      });

      // Add image support if available
      const multimodal = theImage.length > 0;
      
      console.log("Sending request to Perplexity API with messages:", messages);

      // Create the request body as a JSON object
      const requestBody = {
        model: "sonar-pro",
        messages: messages,
        multimodal: multimodal
      };

      console.log("Perplexity API request body:", JSON.stringify(requestBody, null, 2));
      console.log("Perplexity API messages:", messages);

      // Use streaming if available
      isStreaming = true;
      
      try {
        await invoke('stream_perplexity', {
          apiKey: perplexityApiKey,
          requestBody: JSON.stringify(requestBody)
        });
        
        // The streaming response will be handled by the event listeners
        // No need to update streamedGreeting here as it's done in the event listener
        
        // Token metrics will be updated when streaming is complete
      } catch (error) {
        console.error("Error from Perplexity API streaming:", error);
        streamedGreeting += `Error calling Perplexity API: ${error}`;
        isStreaming = false;
      }
    } catch (error) {
      streamedGreeting += `Error: ${error.message || "Unknown error occurred"}`;
      console.error("Error during Perplexity API call:", error);
      isStreaming = false;
    }
  }

  async function loadModels() {
    const ollama = new Ollama({ host: "http://localhost:11434" });

    let models = await ollama.list();
    console.log('models',models);
    let theModelsView = models.models;
    console.log("theModelsView:", theModelsView);
    loadModelNames = theModelsView.map((modelName) => {
      return [
      modelName.name,  
      Utils.formatDate(modelName.modified_at),
      modelName.details.parameter_size,
      modelName.details.quantization_level
        
      ];
    });
    loadModelNames.unshift(
      ["Perplexity - Sonar", "Not local - External API", "N/A", "N/A"],
      ["Fal - Flux", "Not local - External API", "N/A", "N/A"],
      ["Claude 3.7 Sonnet (Latest)", "Not local - External API", "N/A", "N/A"]
    );
    
    //manually add names here
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
      `${theThumbnail != "" ? `<img src="${theThumbnail}" alt="User uploaded image">` : ""}
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

    if (selectedModel === "Perplexity - Sonar") {
      askPerplexity(userMsg);
    } else if (selectedModel === "Fal - Flux") {
      falImage();
    } else if (selectedModel === "Claude 3.7 Sonnet (Latest)") {
      askClaude(userMsg);
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
    <label for="pet-select" class="visualhide">Choose a model:</label>

    <select bind:value={selectedModel} on:change={changeModel}>
      {#each loadModelNames as question}
        <option value={question[0]}>
          {question[0]}
        </option>
      {/each}
    </select>
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

<style lang="">
  @import "./styles.css";
  @import "./darkmode.css";
  @import "./animation.css";
</style>
