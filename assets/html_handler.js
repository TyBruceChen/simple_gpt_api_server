// Display format cleanup for code blocks
(function(){
    function sanitizePre(pre){
        const original = pre.textContent;
        const cleaned = original.replace(/```python\s*/g,'').replace(/```/g,'');
        if(original !== cleaned) pre.textContent = cleaned;
    }

    document.querySelectorAll('pre').forEach(pre=>{
        sanitizePre(pre);
        new MutationObserver(()=>sanitizePre(pre))
            .observe(pre, { childList: true, characterData: true, subtree: true });
    });
})();

// Copy HTML content to clipboard
const htmlToCopy = document.getElementById('answer');
const copyButton = document.getElementById('copyButton');

copyButton.addEventListener('click', async () => {
    try {
        const plainText = htmlToCopy.innerText;
        const htmlContent = htmlToCopy.outerHTML;

        const clipboardItem = new ClipboardItem({
            "text/plain": new Blob([plainText], { type: "text/plain" }),
            "text/html": new Blob([htmlContent], { type: "text/html" })
        });

        await navigator.clipboard.write([clipboardItem]);
        console.log('HTML content copied to clipboard!');
    } catch (err) {
        console.error('Failed to copy HTML: ', err);
    }
});

// Handle form submission (transform into json), receive json response, and display answer
document.getElementById('askForm').addEventListener('submit', async (e)=>{
  e.preventDefault();
  const q = document.getElementById('question').value;
  const model_type = document.getElementById('model_type').value;
  const postData = {question: q,
                    model_type: model_type  // for future use
  };
  document.getElementById('answer').textContent = 'Thinking...';
  try{
    const resp = await fetch('/submit',{method:'POST',headers:{'Content-Type':'application/json'},body:JSON.stringify(postData)});
    const j = await resp.json();
    console.log(j.answer);
    if(j.success) document.getElementById('answer').textContent = j.answer; else document.getElementById('answer').textContent = 'Error: '+(j.error||JSON.stringify(j));
  }catch(err){ document.getElementById('answer').textContent = 'Network error: '+err; }
});