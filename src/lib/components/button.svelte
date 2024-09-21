<script>
	// logic goes here
    export let label='default'
    export let elID
</script>

<!-- markup (zero or more items) goes here -->
<div id='buttonWrap'><button on:click id={elID}>{label}</button></div>

<style>

	

  #buttonWrap {
    padding:2px;
    background-color: var(--buttonbg);
	border-radius: 12rem;
  }
    @keyframes rotateGradient {
	to {
      --border-angle: 1turn;
    }
  }
  
  button {
	/* background-image: linear-gradient(to right, #4776E6 0%, #8E54E9 51%, #4776E6 100%); */
	/* background-image: linear-gradient(to bottom, var(--buttonbg) 0%, var(--buttonbg) 100%); */
  
	background: var(--buttonbg) url('$lib/images/arrow-up.svg') no-repeat .75rem 50%;
	/* box-shadow: inset 0 -2px 8px 0 var(--buttonShadow); */
	background-size: 24px auto;
	font-family: var(--bodyFamily);
	padding: 0rem 1.25rem .1rem 2.5rem;
	min-height: 2.5rem;
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
	transition: all cubic-bezier(.67, -0.04, .31, 1.04) 0.1s;
	position: relative;
	
  
  
  
  }
  
  button:hover {

  }

 
  button:disabled {
    cursor:not-allowed;
  }

  button#stopBtn {
	background: var(--buttonbg) url('$lib/images/stop.svg') no-repeat .75rem 50%;
	
  }




  
  
</style>