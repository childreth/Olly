<script>
  // logic goes here
  export let label = "default";
  export let elID;
</script>

<!-- markup (zero or more items) goes here -->
<div id="buttonWrap" class="button-border button-anime {elID}">
  <button on:click id={elID}>{label}</button>
</div>

<style>
  #buttonWrap {
    position: relative;
    z-index: 0;
	
	
	
  }
  @property --angle {
    syntax: "<angle>";
    initial-value: 0deg;
    inherits: false;
  }
  
  .button-border:after {
    content: "";
    position: absolute;
    height: 100%;
    width: 100%;
    top: 50%;
    left: 50%;
    border-radius: 12rem;
    transform: translate(-50%, -50%);
    background-color: var(--surface);
    z-index: -1;
    padding: 2px;
	transition: background .3s ease-in-out;
	
  }

  #buttonWrap.stopBtn:after {
    background: conic-gradient(
      from var(--angle) at 50% 50%,
      var(--primary) 0% 30%,
      var(--surface) 70% 100%
	  );
    animation: spin 1s linear infinite;
	transition: background .3s ease-in-out;
  }
  @keyframes spin {
    from {
      --angle: 0deg;
    }
    to {
      --angle: 360deg;
    }
  }

  button {
    background: var(--buttonbg) url("$lib/images/arrow-up.svg") no-repeat
      0.75rem 50%;
    /* box-shadow: inset 0 -2px 8px 0 var(--buttonShadow); */
    background-size: 24px auto;
    font-family: var(--bodyFamily);
    padding: 0rem 1.25rem 0.1rem 2.5rem;
    min-height: 2.5rem;
    min-width: 8.25ch;
    /* min-width: 10ch; */
    color: var(--surface);
    font-size: 1.25rem;
    font-weight: 700;
    line-height: 2.75rem;
    border-radius: 12rem;
    display: flex;
    gap: 4px;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    border: none;
    transition: all cubic-bezier(0.67, -0.04, 0.31, 1.04) 0.4s;
  }

  #buttonWrap:hover button{
	background-position: 0.75rem 40%;
  }

  #buttonWrap button#stopBtn {
    background: var(--surface) url("$lib/images/stop.svg") no-repeat 0.75rem
      50%;
	  color: var(--primary);
  }
  #buttonWrap:hover button#stopBtn{
	background-position: 0.75rem 50%;
  }

  
</style>
