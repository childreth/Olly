<script>
  import { invoke } from "@tauri-apps/api/tauri";
  import { Ollama } from "ollama/browser";
  import { onMount } from "svelte";
  import { marked } from "marked";
  import * as Utils from "$lib/utils.js";
  import Button from "$lib/components/button.svelte";

  export let data;
  const modelList = data.modelNames;

  let selectedModel = "llama3.1:latest";
  let activeModel = "";
  let result = "";
  let theImage = [];
  let countConvo = 0;
  let userMsg = "tell me a dad joke";
  let chatResponse = "";
  let streamedGreeting = "";
  $: responseMarked = marked.parse(streamedGreeting);
  let chatConvo = [];
  let tokenSpeed = 0;
  let tokenCount = 0;
  const city = "Westford,MA";
  let name = "Chris";

  let isStreaming = false;
  let abortController = new AbortController();

  const systemMsg = `You are a helpful assistant named 'Olly' who always responds to the user's name ${name}. 
      * Always format the response in markdown using header, lists, paragraphs, text formating. 
      * You can be playful in the response, occasionally add a pun and liberal use of emojis.
      * When diagram charts are requested by the user create them with mermaid.js markdown.
      * If the user asks 'shall we play a game?', always respond with "You know what happen the last time right"'`;

  onMount(async () => {
    const sendBtn = document.querySelector("#sendBtn");
    const imagePreview = document.querySelector("#thumbnails");

    Utils.getCoordinates(city);

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
  });

  async function callOllama() {
    const ollama = new Ollama({ host: "http://localhost:11434" });

    userMsg = document.querySelector("#prompt")?.textContent || "";

    console.log(
      "chatConvo:",
      chatConvo,
      countConvo,
      userMsg,
      systemMsg,
      selectedModel
    );

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
        content: streamedGreeting,
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
      isStreaming = true;
      abortController = new AbortController();

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
        streamedGreeting += `<hr>`;
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
    console.log("model reset");
    countConvo = 0;
    chatConvo = [];
    theImage = [];
    document.querySelector("#thumbnails").innerHTML = "";
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

<header id="title">
  <div id="weather"></div>
  <h1>Olly</h1>
  <!-- <button on:click={sendIt}>SendIt</button> -->
  <div class="input-vertical">
    <label for="pet-select" class="visualhide">Choose a model:</label>

    <select bind:value={selectedModel} on:change={changeModel}>
      {#each modelList as question}
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
  <!-- <div id="settings">
    <h2>Available Models</h2>
    <ul>
      {#each modelList as model}
        <li>{model}</li>
      {/each}
    </ul>
  </div> -->

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
        <!-- <textarea
                      id="prompt"`	wwd
                      name="prompt"
                      placeholder="Ask a question..."
                      >Tell me a dad joke</textarea
                  > -->
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
        >
          <!-- <span>Start with a <mark>3 hour delay</mark> followed by
                      <mark>a conditional split</mark> for SMS -->
        </div>
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
      Model <strong>{selectedModel}</strong>:
      <span class="highlightText">{tokenSpeed} tokens/sec</span>
      &mdash; <span class="highlightText">{tokenCount} total tokens</span>
    </p>
  </div>
</main>

<style lang="">
  @import "./styles.css";
</style>
