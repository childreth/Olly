<script>
  import { invoke } from "@tauri-apps/api/tauri";
  import { Ollama } from "ollama/browser";
  import { onMount } from "svelte";
  import { marked } from "marked";
  import * as Utils from "$lib/utils.js";
  import Button from "$lib/components/button.svelte";
  import Toggle from "$lib/components/toggle.svelte";
  import { appWindow } from "@tauri-apps/api/window";

  import { open } from "@tauri-apps/api/dialog";
  import { confirm } from "@tauri-apps/api/dialog";

  import { fetch, ResponseType } from "@tauri-apps/api/http";

  //basic API call
  const API_URL = "https://rickandmortyapi.com/api/episode";
  let selectedModel = "llama3.1:latest";
  let activeModel = "";
  let result = "";
  let theImage = [];
  let countConvo = 0;
  let userMsg = "tell me a dad joke";
  let lastChatResponse = "";
  let streamedGreeting = "";
  $: responseMarked = marked.parse(streamedGreeting);
  let chatConvo = [];
  let tokenSpeed = 0;
  let tokenCount = 0;
  const city = "Westford,MA";
  let name = "Chris";

  let loadModelNames = [];

  let isStreaming = false;
  let abortController = new AbortController();
  const ollama = new Ollama({ host: "http://localhost:11434" });

  const systemMsg = `You are a helpful assistant named 'Olly' Greet the user ${name}. 
      * Always format the response in markdown using header, lists, paragraphs, text formating. 
      * You can be playful in the response, occasionally add a pun and use of emojis.
      * Always add new return line at the end of the response.
      * If the user asks to play a game, you can choose one of the following games:
        - Tic Tac Toe
        - Chess
        - Falken's Maze
        - Blackjack
        - Checkers
      * Rules for chess:
        - When playing chess, always show a board visual of the chess board
        - The visual for the chess board should be a 2D array of the board
        - The visual for the chess board should be 8x8
        - Reresent chess pieces with appropriate symbols
        - Empty board spaces should be represented with a single underscore '_' 
        - Always place the game visual in pre and code tags
        - Format the visual like this:
          8   ♜  ♞  ♝  ♛  ♚  ♝  ♞  ♜
          7   ♟  ♟  ♟  ♟  ♟  ♟  ♟  ♟
          6   _  _  _  _  _  _  _  _
          5   _  _  _  _  _  _  _  _
          4   _  _  _  _  _  _  _  _
          3   _  _  _  _  _  _  _  _
          2   ♙  ♙  ♙  ♙  ♙  ♙  ♙  ♙
          1   ♖  ♘  ♗  ♕  ♔  ♗  ♘  ♖
              a  b  c  d  e  f  g  h 

      * Rules for Tic Tac Toe:
        - When playing Tic Tac Toe, always show a board visual of the Tic Tac Toe board
        - The visual for the Tic Tac Toe board should be a 3x3 grid, label 1-9 for each space
        - Represent Tic Tac Toe pieces with 'X' and 'O'
        - Empty board spaces should be represented with a single period '.'
        - Always place the game visual in pre and code tags
        - the user is playing as 'X', you are playing as 'O'
        - Format the like this: 
                1 | 2 | 3
                ---------
                4 | 5 | 6
                ---------
                7 | 8 | 9 
      * Rules for Minesweeper:
        - When playing Minesweeper, always show a board visual of the Minesweeper board
        - The visual for the Minesweeper board should be a 2D array of the board
       `;

  
  onMount(async () => {
    const sendBtn = document.querySelector("#sendBtn");
    const imagePreview = document.querySelector("#thumbnails");

    Utils.getCoordinates(city);
    loadModels();

    const fileInput = document.querySelector("#file");

    fileInput.addEventListener("change", (e) => {
      const file = fileInput.files[0];
      const reader = new FileReader();

      reader.addEventListener("load", () => {
        //uploadedImg = reader.result;
        let uploadedImg = reader.result.split(",")[1];
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




  async function loadModels() {
    const ollama = new Ollama({ host: "http://localhost:11434" });

    let models = await ollama.list();
    let theModelsView = models.models;

    loadModelNames = theModelsView.map((modelName) => {
      return modelName.name;
    });
    loadModelNames.unshift("Fal - Flux");
    console.log("loadModelNames:", loadModelNames);
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
    streamedGreeting += `<h2 class="userMsg"> ${userMsg} </h2>`;
    streamedGreeting += `<p><small><strong>${selectedModel}</strong></small></p>`
    document.querySelector("#prompt").textContent = "";

    

    //add user message to the thread
    if (countConvo == 0) {
      chatConvo[countConvo++] = { role: "system", content: systemMsg };
      chatConvo[countConvo++] = {
        role: "user",
        content: userMsg,
        images: theImage,
      };
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

    if (selectedModel === "Fal - Flux") {
      falImage();
    } else if (selectedModel === "Canary Chrome") {
      canaryChrome();
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
</script>

<div id="settings">
  <div class="settings-content">
    <header>
      <h2>Manage models</h2><button class="icon" on:click={Utils.closeSettings}><svg xmlns="http://www.w3.org/2000/svg" height="24px" viewBox="0 -960 960 960" width="24px" fill="currentColor"><path d="M480-437.85 277.08-234.92q-8.31 8.3-20.89 8.5-12.57.19-21.27-8.5-8.69-8.7-8.69-21.08 0-12.38 8.69-21.08L437.85-480 234.92-682.92q-8.3-8.31-8.5-20.89-.19-12.57 8.5-21.27 8.7-8.69 21.08-8.69 12.38 0 21.08 8.69L480-522.15l202.92-202.93q8.31-8.3 20.89-8.5 12.57-.19 21.27 8.5 8.69 8.7 8.69 21.08 0 12.38-8.69 21.08L522.15-480l202.93 202.92q8.3 8.31 8.5 20.89.19 12.57-8.5 21.27-8.7 8.69-21.08 8.69-12.38 0-21.08-8.69L480-437.85Z"/></svg></button>
    </header>
   
    <ul>
      {#each loadModelNames as model}
        <li>{model} <button class='basic delete' on:click={deleteModel(`${model}`)}>Delete</button></li>
      {/each}
    </ul>
  </div>
  
</div>

<header id="title">
  <div id="weather">
    <span class="weather-icon"></span>
    <div><span class="weather-report"></span></div>
    <div class="weather-details"></div>
  </div>
  <!-- <button on:click={confirmDialog}>Show Dialog</button> -->
  <h1 on:click={Utils.toggleTheme}>Olly</h1>
  <!-- <button class="basic" on:click={Utils.toggleTheme}>Test it</button> -->
  <div class="input-vertical">
    <label for="pet-select" class="visualhide">Choose a model:</label>

    <select bind:value={selectedModel} on:change={changeModel}>
      {#each loadModelNames as question}
        <option value={question}>
          {question}
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
        <Button
          label={isStreaming ? "Stop" : "Start"}
          on:click={isStreaming ? stopStreaming : callOllama}
          elID={isStreaming ? "stopBtn" : "sendBtn"}
        />
      </div>
    </div>
    <!-- <audio id="speech" controls style="position: fixed; bottom: 0; left: 0; width: 100%;" /> -->
    <p class="modelInfo">
      <a class='basic' on:click={Utils.openSettings}>Manage models</a> &nbsp; | &nbsp; <strong>{selectedModel}</strong>:
      <span class="highlightText">{tokenSpeed} tokens/sec</span>
      &mdash; <span class="highlightText">{tokenCount} total tokens</span> 
      
    </p>
  </div>
</main>

<style lang="">
  @import "./styles.css";
  @import "./darkmode.css";
</style>
