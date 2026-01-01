<script>
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { Ollama } from "ollama/browser";
  import { onMount } from "svelte";
  import { marked } from "marked";
  import { fly } from "svelte/transition";
  import * as Utils from "$lib/utils.js";
  import { tools, executeTool, supportsToolCalling } from "$lib/tools.js";
  import SendButton from "$lib/components/sendButton.svelte";
  import Button from "$lib/components/button.svelte";
  import Select from "$lib/components/select.svelte";
  import ColorPicker from "$lib/components/colorPicker.svelte";
  import Toggle from "$lib/components/darkModeToggle.svelte";
  import Toast from "$lib/components/toast.svelte";
  import SearchableSelect from "$lib/components/searchableSelect.svelte";
  import TabbedModelManager from "$lib/components/tabbedModelManager.svelte";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  

  import { open, confirm } from "@tauri-apps/plugin-dialog";

  import { fetch } from "@tauri-apps/plugin-http";
  // Use this to get the environment variable


  //basic API call for testing tools
  const API_URL = "https://rickandmortyapi.com/api/episode";
  let selectedModel = "smollm2:1.7b";
  let selectedModelOption = null;
  let activeModel = "";
  let result = "";
  let theImage= [];
  let theThumbnail = "";
  let imageMediaType = "";
  let isProcessingImage = false;
  let countConvo = 0;
  let userMsg = "tell me a dad joke";
  let lastChatResponse = "";
  let streamedGreeting = "";
  let responseMarked = "";
  let chatConvo = [];
  let tokenSpeed = 0;
  let tokenCount = 0;
  const city = "Westford,MA";
  let name = "Craig";
  let loadModelNames = [];
  let allModels = [
    // Initialize with the currently selected model so it shows in dropdown immediately
    { id: "smollm2:1.7b", name: "smollm2:1.7b", description: "Loading models...", provider: "ollama" }
  ];
  let isStreaming = false;
  let abortController = new AbortController();
  const ollama = new Ollama({ host: "http://localhost:11434" });
  const appWindow = getCurrentWindow();

  let darkMode = false;
  let themeColor = "#000099"; // default color
  
  // Toast notification state
  let toastVisible = false;
  let toastMessage = "";
  let toastType = "info";

//very basic system prompt to test speed vs terimal interface
  const systemMsg = `You are a helpful assistant and like really responsing with emojies. You job will be to help write better content for the user. Always return markdown formatted responses.`;

  
  onMount(async () => {
    console.log('=== APP MOUNTED ===');

    // Configure marked for proper markdown rendering
    marked.setOptions({
      breaks: true,
      gfm: true,
      pedantic: false
    });

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

    // Test Ollama command directly
    try {
      console.log('=== TESTING get_ollama_models COMMAND ===');
      const testModels = await invoke("get_ollama_models");
      console.log('Direct test result:', testModels);
    } catch (error) {
      console.error('Direct test failed:', error);
    }

    console.log('=== CALLING loadModels() ===');
    await loadModels();
    console.log('=== loadModels() COMPLETE ===');
    console.log('Final allModels array:', allModels);

    const fileInput = document.querySelector("#file");

    prompt.addEventListener("keydown", (e) => {
      if (e.key === "Enter") {
        callOllama()
      }
    });

    fileInput.addEventListener("change", async (e) => {
      const file = fileInput.files[0];
      if (!file) return;

      // Show processing indicator
      isProcessingImage = true;

      try {
        // Use the new compression utility
        const compressed = await Utils.compressImage(file);

        // Store compressed base64 and media type
        theImage.push(compressed.base64);
        imageMediaType = compressed.mediaType;
        theThumbnail = compressed.thumbnail;

        // Display thumbnail in UI
        let thumbnail = document.createElement("img");
        thumbnail.src = compressed.thumbnail;
        imagePreview?.appendChild(thumbnail);

        console.log("Image compressed and ready:", {
          originalSize: file.size,
          compressedSize: compressed.base64.length,
          mediaType: compressed.mediaType
        });
      } catch (error) {
        console.error("Error processing image:", error);
        toastMessage = "Failed to process image. Please try again.";
        toastType = "error";
        toastVisible = true;
      } finally {
        isProcessingImage = false;
      }
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
      responseMarked = marked.parse(streamedGreeting);
    });

    appWindow.listen('claude-stream-done', (event) => {
      console.log("Claude streaming completed");
      isStreaming = false;
      responseMarked = marked.parse(streamedGreeting);
      Utils.addCopyButtonToPre();
    });

    // Perplexity streaming event listeners
    appWindow.listen('perplexity-stream', (event) => {
      const content = event.payload;
      streamedGreeting += content;
      lastChatResponse += content;
      responseMarked = marked.parse(streamedGreeting);
    });

    appWindow.listen('perplexity-stream-done', (event) => {
      console.log("Perplexity streaming completed");
      const data = event.payload;

      // Add citations if present
      if (data.citations && data.citations.length > 0) {
        let citationsHtml = '\n\n---\n\n### References\n\n';
        data.citations.forEach((/** @type {string} */ url, /** @type {number} */ index) => {
          citationsHtml += `${index + 1}. [${url}](${url})\n`;
        });
        streamedGreeting += citationsHtml;
        lastChatResponse += citationsHtml;
      }

      isStreaming = false;
      responseMarked = marked.parse(streamedGreeting);
      Utils.addCopyButtonToPre();
    });
    
    // Initialize feather icons
    if (typeof window !== 'undefined' && window.feather) {
      window.feather.replace();
    }

    // Add scroll listener to chat container
    const chatContainer = document.querySelector("#chat-container");
    if (chatContainer) {
      chatContainer.addEventListener('scroll', handleScroll);
    }

    //callOllama()
  });

  async function rickAndMorty() {
    const response = await fetch(API_URL, {
      method: "GET"
    });
    const data = await response.json();
    console.log("response", data);
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

      // Build conversation history for Claude
      let claudeMessages = [];

      if (countConvo == 0) {
        // First message - check if we have images
        if (theImage.length > 0) {
          // Multimodal message with images
          let content = [];

          // Add images first
          theImage.forEach(base64Data => {
            content.push({
              type: "image",
              source: {
                type: "base64",
                media_type: imageMediaType || "image/jpeg",
                data: base64Data
              }
            });
          });

          // Add text content
          content.push({
            type: "text",
            text: userMsg
          });

          claudeMessages = [{ role: "user", content: content }];
        } else {
          // Text-only message
          claudeMessages = [{ role: "user", content: userMsg }];
        }
      } else {
        // Build full conversation history from chatConvo
        claudeMessages = chatConvo.filter(msg => msg.role !== "system").map(msg => {
          // Convert stored messages to proper format
          if (msg.images && msg.images.length > 0) {
            let content = [];
            msg.images.forEach(base64Data => {
              content.push({
                type: "image",
                source: {
                  type: "base64",
                  media_type: imageMediaType || "image/jpeg",
                  data: base64Data
                }
              });
            });
            content.push({
              type: "text",
              text: msg.content
            });
            return { role: msg.role, content: content };
          } else {
            return { role: msg.role, content: msg.content };
          }
        });

        // Add current user message with images if present
        if (theImage.length > 0) {
          let content = [];
          theImage.forEach(base64Data => {
            content.push({
              type: "image",
              source: {
                type: "base64",
                media_type: imageMediaType || "image/jpeg",
                data: base64Data
              }
            });
          });
          content.push({
            type: "text",
            text: userMsg
          });
          claudeMessages.push({ role: "user", content: content });
        } else {
          claudeMessages.push({ role: "user", content: userMsg });
        }
      }

      await invoke('stream_claude', {
        model: selectedModel,
        prompt: userMsg,
        messages: claudeMessages
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
      // Get all models from backend (don't fail if this errors)
      let backendModels = [];
      try {
        backendModels = await invoke("get_all_models");
        console.log('Backend models:', backendModels);
      } catch (error) {
        console.warn("Failed to get backend models:", error);
      }

      // Get Ollama models if available
      let ollamaModels = [];
      try {
        console.log('Attempting to fetch Ollama models from backend...');
        ollamaModels = await invoke("get_ollama_models");
        console.log('Ollama models response:', ollamaModels);

        if (ollamaModels && ollamaModels.length > 0) {
          // Keep old format for settings display
          loadModelNames = ollamaModels.map((model) => [
            model.name,
            model.details?.modified_at || "Unknown",
            model.details?.parameter_size || "Unknown",
            model.details?.quantization_level || "Unknown"
          ]);
          console.log(`Successfully loaded ${ollamaModels.length} Ollama models`);
        } else {
          console.warn('Ollama returned empty models list');
          loadModelNames = [];
        }
      } catch (error) {
        console.warn("Ollama not available:", error);
        loadModelNames = [];
      }

      // Get Claude models from backend
      let claudeModels = [];
      let claudeApiSuccess = false;
      try {
        console.log('Fetching Claude models from backend...');
        const claudeResponse = await invoke("get_claude_models");
        console.log('Claude models response:', claudeResponse);
        
        if (claudeResponse && claudeResponse.length > 0) {
          claudeModels = claudeResponse;
          // Add to old format for settings display
          const claudeModelNames = claudeResponse.map(model => [
            model.name,
            "Not local - External API",
            "N/A",
            "N/A"
          ]);
          loadModelNames.push(...claudeModelNames);
          claudeApiSuccess = true;
          console.log(`Successfully loaded ${claudeModels.length} Claude models`);
        } else {
          console.log('No Claude models returned from backend');
        }
      } catch (error) {
        console.error("Failed to get Claude models:", error);
      }

      // Add external API models to old format for settings (keeping fallback)
      loadModelNames.unshift(["Fal - Flux","Not local - External API","N/A","N/A"]);
      if (!claudeApiSuccess) {
        console.log('No Claude models found, adding fallback');
        loadModelNames.unshift(["Claude 3.5 Sonnet","Not local - External API","N/A","N/A"]);
      }
      loadModelNames.push(...backendModels.filter(m => m.provider === "perplexity").map(m => [m.name, "Not local - External API", "N/A", "N/A"]));

      // Combine all models for the searchable select
      allModels = [...backendModels, ...ollamaModels, ...claudeModels];
      console.log('All models combined:', allModels);
      console.log('Total models loaded:', allModels.length);

      // If no models were loaded at all, use fallback
      if (allModels.length === 0) {
        console.warn('No models loaded from any provider, using fallback');
        allModels = [
          { id: "claude-3-7-sonnet-20250219", name: "Claude 3.7 Sonnet", description: "Most capable Claude model", provider: "claude" },
          { id: "fal-flux", name: "Fal - Flux", description: "Image generation model", provider: "fal" },
          { id: "sonar-pro", name: "Sonar Pro", description: "Perplexity search model", provider: "perplexity" }
        ];
      }

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
    // Reset scroll override for new message
    userScrolledUp = false;

    userMsg = document.querySelector("#prompt").textContent || "";
    const messageId = Date.now(); // Unique ID for this message

    //add user message to the top of the chat
    streamedGreeting += `<h2 class="userMsg" data-msg-id="${messageId}"> <span>${userMsg}</span>` +
      `${theThumbnail ? `<img src="${theThumbnail}" alt="User uploaded image">` : ""}
      </h2>`;


    streamedGreeting += `<p><small><strong>${selectedModel}</strong></small></p>`;
    streamedGreeting += "\n\n";

    // Update responseMarked immediately so user sees their message
    responseMarked = marked.parse(streamedGreeting);

    // Apply animation only to the new message, once
    requestAnimationFrame(() => {
      const newMsg = document.querySelector(`[data-msg-id="${messageId}"]`);
      if (newMsg) {
        newMsg.classList.add('animate-in');
      }
    });

    document.querySelector("#prompt").textContent = "";
    document.querySelector("#thumbnails").innerHTML = "";

    // Reset file input to allow selecting the same file again
    const fileInput = document.querySelector("#file");
    if (fileInput) fileInput.value = "";

    //add user message to the thread
    if (countConvo == 0) {
      chatConvo[countConvo++] = { role: "system", content: systemMsg };
      chatConvo[countConvo++] = {
        role: "user",
        content: userMsg,
        images: [...theImage], // Clone array to preserve in history
        mediaType: imageMediaType
      };
    } else {
      chatConvo[countConvo++] = {
        role: "assistant",
        content: lastChatResponse,
        images: [],
      };
      chatConvo[countConvo++] = {
        role: "user",
        content: userMsg,
        images: [...theImage], // Clone array to preserve in history
        mediaType: imageMediaType
      };
    }

    // Clear image arrays AFTER adding to conversation
    theImage = [];
    theThumbnail = "";
    imageMediaType = "";

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

      // Check if model supports tool calling
      // Note: Tool calling (especially calendar) may not work in dev mode due to missing Info.plist bundle
      const useTools = supportsToolCalling(selectedModel);
      console.log(`🔧 Tool calling ${useTools ? 'ENABLED' : 'DISABLED'} for model: ${selectedModel}`);

      try {
        // Agent loop for tool calling
        let continueLoop = true;
        let loopCount = 0;
        const maxLoops = 5; // Prevent infinite loops

        while (continueLoop && loopCount < maxLoops) {
          loopCount++;
          console.log(`🔄 Agent loop iteration ${loopCount}`);

          const response = await ollama.chat({
            model: selectedModel,
            messages: chatConvo,
            stream: true,
            tools: useTools ? tools : undefined,
            options: {
              temperature: 0.9,
            },
            signal: abortController.signal,
          });

          let currentMessage = { role: 'assistant', content: '' };
          let toolCalls = [];

          for await (const part of response) {
            if (abortController.signal.aborted) {
              ollama.abort();
              break;
            }

            // Handle tool calls
            if (part.message.tool_calls) {
              toolCalls = part.message.tool_calls;
              console.log('🔧 Tool calls detected:', toolCalls);
            }

            // Stream content
            if (part.message.content) {
              streamedGreeting += part.message.content;
              lastChatResponse += part.message.content;
              currentMessage.content += part.message.content;
              responseMarked = marked.parse(streamedGreeting);
            }

            // Track tokens
            if (part.eval_count) {
              tokenCount = Number(part.eval_count);
              tokenSpeed = Number(
                (part.eval_count / part.eval_duration) * Math.pow(10, 9)
              ).toFixed(2);
            }
          }

          // If there are tool calls, execute them
          if (toolCalls && toolCalls.length > 0) {
            console.log(`🔧 Executing ${toolCalls.length} tool(s)...`);
            
            // Add tool call indicator to UI
            streamedGreeting += `\n\n*🔍 Using tools: ${toolCalls.map(tc => tc.function.name).join(', ')}...*\n\n`;
            responseMarked = marked.parse(streamedGreeting);

            // Add assistant message with tool calls to conversation
            chatConvo[countConvo++] = {
              role: 'assistant',
              content: currentMessage.content,
              tool_calls: toolCalls
            };

            // Execute each tool and add results to conversation
            for (const toolCall of toolCalls) {
              try {
                const toolName = toolCall.function.name;
                const toolArgs = toolCall.function.arguments;
                
                console.log(`🔧 Calling ${toolName} with args:`, toolArgs);
                const result = await executeTool(toolName, toolArgs);
                console.log(`✅ Tool ${toolName} result:`, result);

                // Add tool result to conversation
                chatConvo[countConvo++] = {
                  role: 'tool',
                  content: result,
                  tool_name: toolName
                };

                // Show tool result in UI (optional, can be removed for cleaner output)
                // streamedGreeting += `\n*Tool ${toolName} completed*\n`;
                // responseMarked = marked.parse(streamedGreeting);

              } catch (error) {
                console.error(`❌ Tool execution error:`, error);
                chatConvo[countConvo++] = {
                  role: 'tool',
                  content: JSON.stringify({ error: error.toString() }),
                  tool_name: toolCall.function.name
                };
              }
            }

            // Continue loop to get final response with tool results
            continueLoop = true;
            lastChatResponse = ""; // Reset for next iteration
          } else {
            // No tool calls, we're done
            continueLoop = false;
          }
        }

        if (loopCount >= maxLoops) {
          console.warn('⚠️ Max agent loops reached');
          streamedGreeting += `\n\n*Maximum tool iterations reached.*\n`;
          responseMarked = marked.parse(streamedGreeting);
        }

        console.log("streamGreet:", streamedGreeting);
      } catch (error) {
        if (error.name === "AbortError") {
          console.log("Stream was aborted");
        } else {
          console.error("Error during streaming:", error);
          streamedGreeting += `\n\n*Error: ${error.message}*\n`;
          responseMarked = marked.parse(streamedGreeting);
        }
      } finally {
        isStreaming = false;
        responseMarked = marked.parse(streamedGreeting);
        Utils.addCopyButtonToPre();
      }
    }

    // sendBtn.disabled = false;
    // sendBtn.textContent = "Send";
  }

  let scrollScheduled = false;
  let userScrolledUp = false;
  let showScrollButton = false;
  let scrollCheckTimeout = null;

  function autoScroll() {
    const chatContainer = document.querySelector("#chat-container");
    if (chatContainer) {
      // Use instant scroll during streaming for better performance
      chatContainer.scrollTo({
        top: chatContainer.scrollHeight,
        behavior: isStreaming ? 'instant' : 'smooth'
      });
    }
    scrollScheduled = false;
  }

  function scheduleScroll() {
    if (!scrollScheduled) {
      scrollScheduled = true;
      requestAnimationFrame(autoScroll);
    }
  }

  function handleScroll(event) {
    const chatContainer = event.target;
    const scrollBottom = chatContainer.scrollHeight - chatContainer.scrollTop;
    const isAtBottom = scrollBottom <= chatContainer.clientHeight + 150;
    const hasOverflow = chatContainer.scrollHeight > chatContainer.clientHeight;

    // Debounce button visibility to prevent flickering
    if (scrollCheckTimeout) {
      clearTimeout(scrollCheckTimeout);
    }

    scrollCheckTimeout = setTimeout(() => {
      showScrollButton = hasOverflow && !isAtBottom;
    }, 100);

    // Track if user is scrolled up during streaming
    if (isStreaming && !isAtBottom) {
      userScrolledUp = true;
    } else if (isAtBottom) {
      userScrolledUp = false;
    }
  }

  // Don't auto-scroll anymore - user controls scrolling manually
  // $: if (responseMarked && !isStreaming) {
  //   scheduleScroll();
  // }

  // Update button visibility when content changes (debounced)
  $: if (responseMarked) {
    if (scrollCheckTimeout) {
      clearTimeout(scrollCheckTimeout);
    }

    scrollCheckTimeout = setTimeout(() => {
      requestAnimationFrame(() => {
        const chatContainer = document.querySelector("#chat-container");
        if (chatContainer) {
          const scrollBottom = chatContainer.scrollHeight - chatContainer.scrollTop;
          const isAtBottom = scrollBottom <= chatContainer.clientHeight + 150;
          const hasOverflow = chatContainer.scrollHeight > chatContainer.clientHeight;
          showScrollButton = hasOverflow && !isAtBottom;
        }
      });
    }, 100);
  }

  function changeModel() {
    //reset the chat for new conversation+model
    console.log("model reset");
    countConvo = 0;
    chatConvo = [];
    lastChatResponse = "";
    theImage = [];
    theThumbnail = "";
    imageMediaType = "";
    tokenCount = 0;
    tokenSpeed = 0;
    document.querySelector("#thumbnails").innerHTML = "";

    // Reset file input
    const fileInput = document.querySelector("#file");
    if (fileInput) fileInput.value = "";

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

<svelte:head>
  <script src="/src/lib/feather.min.js"></script>
</svelte:head>

<div id="settings">
  <div class="settings-content">
    <header>
      <h1 class='text-xl'>Settings</h1><Button type="icon-only" icon="x" on:click={Utils.closeSettings}/>
    </header>

    <section>
      <h2 class='text-lg'>General</h2>
      <p style='display:flex;flex-direction:row;align-items:flex-end;gap:2.5rem;'>

        <!-- <Select id='typeface' small={true} /> -->
        <Toggle  id="darkModeToggle"  />
        <ColorPicker color={themeColor} on:colorChange={handleColorChange} />
      </p>
      <TabbedModelManager 
        {loadModelNames} 
        onModelDeleted={loadModels}
      />
  </section>
  </div>
  
</div>

<header id="title">
  <div class='leftCol'>
 
    <div id="weather">
      <span class="weather-icon"></span>
      <div><span class="weather-report"></span></div>
     <!-- <div class="weather-details"></div> -->
    
    </div>
    <span>|</span>
    <Button label="Settings" type="link" on:click={Utils.openSettings} />
  </div>
  <h1>Olly</h1>
  <div class="rightCol">
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

    {#if showScrollButton}
      <button
        class="scroll-to-bottom scroll-button-enter"
        aria-label="Scroll to bottom of chat"
        in:fly={{ y: 4, duration: 200 }}
        out:fly={{ y: 4, duration: 200 }}
        on:click={() => {
        const chatContainer = document.querySelector("#chat-container");
        if (chatContainer) {
          chatContainer.scrollTo({
            top: chatContainer.scrollHeight,
            behavior: 'smooth'
          });
          userScrolledUp = false;
          showScrollButton = false;
        }
      }}>
       <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="feather feather-arrow-down"><line x1="12" y1="5" x2="12" y2="19"></line><polyline points="19 12 12 19 5 12"></polyline></svg>
      </button>
    {/if}
  </div>
 

  <div id="userinput" class="halftone">
    <div class="combotext">
      <div id="imgInput">
        <label for="file" class="custom-file-upload"
          ><span class="visualhide">Upload an image</span></label
        >
        <input type="file" id="file" accept="image/jpeg,image/png,image/jpg" />
        {#if isProcessingImage}
          <span class="processing-indicator">Processing image...</span>
        {/if}
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
          label={isStreaming ? "Stop" : "Send"}
          on:click={isStreaming ? stopStreaming : callOllama}
          elID="{isStreaming ? "stopBtn" : "sendBtn"}"
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

<Toast bind:visible={toastVisible} message={toastMessage} type={toastType} />

<style lang="">
  @import "./styles.css";
  @import "./darkmode.css";
  @import "./animation.css";
</style>