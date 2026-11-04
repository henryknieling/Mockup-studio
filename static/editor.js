(() => {
  const root = document.querySelector('#editor');
  if (!root) return;
  const canvas = document.querySelector('#canvas'), device = document.querySelector('#device');
  const tabs = [...document.querySelectorAll('.screen-tab')];
  const state = { screen: root.dataset.screen, project: root.dataset.project, width: 390, height: 844, frame: 'phone', elements: [], selected: null, history: [], future: [], saveTimer: null, drag: null };
  const $ = (s) => document.querySelector(s);
  const esc = (s) => String(s ?? '').replace(/[&<>"']/g, c => ({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&#39;'}[c]));
  const clone = (x) => JSON.parse(JSON.stringify(x));
  const selected = () => state.elements.find(e => e.id === state.selected);
  const isNavigation = (e) => e.kind === 'nav' || e.kind === 'web-nav';
  function navItems(e) {
    if (!Array.isArray(e.items) || e.items.length === 0) {
      const defaults = e.kind === 'web-nav' ? ['Logo','Products','About','Contact'] : ['Home','Search','Profile'];
      e.items = e.text ? e.text.split(/\u3000+|\s{2,}/).map(label => label.trim()).filter(Boolean) : defaults;
      if (!e.items.length) e.items = defaults;
    }
    return e.items;
  }
  function syncNavText(e) { e.text = navItems(e).join('　'); }
  function textJustification(e) {
    return ({left:'flex-start',center:'center',right:'flex-end','space-between':'space-between'}[e.text_align] || 'flex-start');
  }
  const saveHistory = () => { state.history.push(clone(state.elements)); if (state.history.length > 80) state.history.shift(); state.future = []; updateHistoryButtons(); };
  const updateHistoryButtons = () => { $('#undo').disabled = !state.history.length; $('#redo').disabled = !state.future.length; };
  const status = (text) => { $('#save-status').textContent = text; };

  async function loadScreen(id, tab) {
    state.screen = id; root.dataset.screen = id;
    const r = await fetch(`/api/screens/${id}`); if (!r.ok) return;
    const data = await r.json(); state.width = data.width; state.height = data.height; state.frame = data.frame;
    state.elements = data.document?.elements || []; state.selected = null; state.history = []; state.future = [];
    $('#active-name').textContent = data.name; $('#rename-screen [name=name]').value = data.name;
    $('#viewport-form [name=width]').value = state.width; $('#viewport-form [name=height]').value = state.height; $('#viewport-form [name=frame]').value = state.frame;
    $('#viewport-label').textContent = `${state.width} × ${state.height}`;
    $('#screen-export').onclick = () => { location.href = `/export/screens/${id}`; };
    tabs.forEach(t => t.classList.toggle('active', t === tab));
    render(); updateHistoryButtons();
  }

  function render() {
    canvas.style.width = `${state.width}px`; canvas.style.height = `${state.height}px`;
    device.className = `device-frame ${state.frame}`;
    $('#viewport-label').textContent = `${state.width} × ${state.height}`;
    canvas.replaceChildren();
    canvas.classList.toggle('phone-screen', state.frame === 'phone');
    if (state.frame === 'phone') {
      const statusBar = document.createElement('div');
      statusBar.className = 'phone-status';
      statusBar.innerHTML = '<span>9:41</span><span class="phone-status-icons">●●●　◕　▰</span>';
      canvas.append(statusBar);
    }
    state.elements.forEach(e => {
      const el = document.createElement('div'); el.className = `canvas-element element-${e.kind}${state.selected === e.id ? ' selected' : ''}`;
      el.dataset.id = e.id; Object.assign(el.style, {left:`${e.x}px`,top:`${e.y}px`,width:`${e.width}px`,height:`${e.height}px`,background:e.kind === 'text' || e.kind === 'heading' ? 'transparent' : e.fill,color:e.color,borderRadius:`${e.radius}px`,fontSize:`${e.font_size}px`,textAlign:e.text_align||'left',justifyContent:(e.kind==='text'||e.kind==='heading'||e.kind==='nav-title'||isNavigation(e))?textJustification(e):''});
      if (e.kind === 'image') el.innerHTML = `<span class="image-symbol">▧</span><span>${esc(e.text || 'Image placeholder')}</span>`;
      else if (e.kind === 'input') el.innerHTML = `<span class="placeholder">${esc(e.placeholder || e.text || 'Enter text…')}</span>`;
      else if (isNavigation(e)) el.innerHTML = navItems(e).map((label,index)=>`<span class="nav-tab-item${index===0?' active':''}">${esc(label)}</span>`).join('');
      else if (e.kind === 'nav-title') el.textContent = e.text || 'Title';
      else el.textContent = e.text || (e.kind === 'button' ? 'Button' : e.kind === 'card' || e.kind === 'web-card' ? 'Card content' : e.kind === 'heading' ? 'Heading' : 'Text');
      el.addEventListener('pointerdown', startDrag);
      el.addEventListener('click', ev => { ev.stopPropagation(); state.selected = e.id; render(); });
      if (state.selected === e.id) { const handle=document.createElement('i'); handle.className='resize-handle'; handle.addEventListener('pointerdown', startResize); el.append(handle); }
      canvas.append(el);
    });
    renderInspector(); renderLayers(); fitCanvas();
  }
  function fitCanvas() {
    const stage = document.querySelector('.stage-wrap'); if (!stage) return;
    const scale = Math.min(1, (stage.clientWidth - 80) / state.width, (stage.clientHeight - 60) / state.height);
    canvas.style.transform = `scale(${scale})`; canvas.style.transformOrigin = 'top left';
    device.style.width = `${state.width * scale}px`; device.style.height = `${state.height * scale}px`;
    device.style.setProperty('--scale', scale); $('#zoom-label').textContent = `${Math.round(scale*100)}%`;
  }
  function renderLayers() {
    $('#layer-count').textContent = state.elements.length;
    $('#layers-list').innerHTML = state.elements.map((e,i)=>`<div class="layer-row ${e.id===state.selected?'active':''}" data-id="${esc(e.id)}"><span class="layer-glyph">${({text:'T',heading:'H',button:'▰',image:'▧',input:'▱',card:'▢','web-card':'▤',nav:'☰','web-nav':'↔','nav-title':'↕'})[e.kind]||'◇'}</span><button class="layer-select">${esc(e.text || e.kind)}</button><button class="layer-order" data-dir="up" title="Move up" ${i===state.elements.length-1?'disabled':''}>↑</button><button class="layer-order" data-dir="down" title="Move down" ${i===0?'disabled':''}>↓</button></div>`).join('');
    document.querySelectorAll('.layer-row').forEach(row => row.querySelector('.layer-select').onclick = () => { state.selected=row.dataset.id; render(); });
    document.querySelectorAll('.layer-order').forEach(btn => btn.onclick=()=>{const row=btn.closest('.layer-row'),i=state.elements.findIndex(x=>x.id===row.dataset.id),j=i+(btn.dataset.dir==='up'?1:-1);if(j<0||j>=state.elements.length)return;saveHistory();[state.elements[i],state.elements[j]]=[state.elements[j],state.elements[i]];render();scheduleSave();});
  }
  function renderInspector() {
    const e=selected(), panel=$('#inspector-content');
    if(!e){panel.innerHTML='<div class="empty-inspector">Select an element on the canvas to edit its properties.</div>';return;}
    const alignment = ['heading','text','nav-title','nav','web-nav'].includes(e.kind) ? `<label class="field">${isNavigation(e)?'Tab alignment':'Text alignment'}<select data-prop="text_align"><option value="left" ${(!e.text_align||e.text_align==='left')?'selected':''}>Left</option><option value="center" ${e.text_align==='center'?'selected':''}>Center</option><option value="right" ${e.text_align==='right'?'selected':''}>Right</option>${isNavigation(e)?`<option value="space-between" ${e.text_align==='space-between'?'selected':''}>Space evenly</option>`:''}</select></label>` : '';
    const navEditor = isNavigation(e) ? `<div class="nav-items-editor"><span class="nav-editor-heading">Tab labels</span>${navItems(e).map((label,index)=>`<div class="nav-item-edit"><input data-nav-item="${index}" value="${esc(label)}" aria-label="Tab ${index+1} label"><button type="button" data-remove-nav-item="${index}" title="Remove tab" ${navItems(e).length===1?'disabled':''}>×</button></div>`).join('')}<button type="button" id="add-nav-item" class="add-nav-item">＋ Add tab</button></div>` : `<label class="field">Content<input data-prop="text" value="${esc(e.text)}" placeholder="${esc(e.placeholder||'Text')}"></label>`;
    panel.innerHTML=`<div class="selected-type"><span>${esc(e.kind.replace('-', ' '))}</span><button id="remove-element" title="Delete element">⌫</button></div>${navEditor}${alignment}${e.kind==='input'?`<label class="field">Placeholder<input data-prop="placeholder" value="${esc(e.placeholder)}"></label>`:''}<div class="field-grid"><label class="field">X<input type="number" data-prop="x" value="${Math.round(e.x)}"></label><label class="field">Y<input type="number" data-prop="y" value="${Math.round(e.y)}"></label><label class="field">Width<input type="number" data-prop="width" value="${Math.round(e.width)}"></label><label class="field">Height<input type="number" data-prop="height" value="${Math.round(e.height)}"></label></div><div class="field-grid"><label class="field">Fill<input type="color" data-prop="fill" value="${/^#[0-9a-f]{6}$/i.test(e.fill)?e.fill:'#ffffff'}"></label><label class="field">Text color<input type="color" data-prop="color" value="${/^#[0-9a-f]{6}$/i.test(e.color)?e.color:'#172033'}"></label></div><label class="field">Font size <span class="range-value">${e.font_size}px</span><input type="range" min="10" max="48" data-prop="font_size" value="${e.font_size}"></label>`;
    panel.querySelector('#remove-element').onclick=()=>removeElement(e.id);
    panel.querySelectorAll('[data-prop]').forEach(input=>{input.addEventListener('focus',()=>saveHistory());input.addEventListener('input',()=>{e[input.dataset.prop]=['x','y','width','height','font_size'].includes(input.dataset.prop)?Number(input.value):input.value;if(input.dataset.prop==='font_size')input.previousElementSibling.querySelector('.range-value')?.replaceChildren(`${e.font_size}px`);renderCanvasOnly();scheduleSave();});});
    panel.querySelectorAll('[data-nav-item]').forEach(input=>{input.addEventListener('focus',()=>saveHistory());input.addEventListener('input',()=>{navItems(e)[Number(input.dataset.navItem)]=input.value;syncNavText(e);renderCanvasOnly();scheduleSave();});});
    panel.querySelectorAll('[data-remove-nav-item]').forEach(button=>button.onclick=()=>{const items=navItems(e);if(items.length<2)return;saveHistory();items.splice(Number(button.dataset.removeNavItem),1);syncNavText(e);render();scheduleSave();});
    panel.querySelector('#add-nav-item')?.addEventListener('click',()=>{saveHistory();navItems(e).push(`Tab ${navItems(e).length+1}`);syncNavText(e);render();scheduleSave();});
  }
  function renderCanvasOnly(){
    const e=selected(),node=e&&canvas.querySelector(`[data-id="${CSS.escape(e.id)}"]`);if(!e||!node)return;
    Object.assign(node.style,{left:`${e.x}px`,top:`${e.y}px`,width:`${e.width}px`,height:`${e.height}px`,background:e.kind==='text'||e.kind==='heading'?'transparent':e.fill,color:e.color,borderRadius:`${e.radius}px`,fontSize:`${e.font_size}px`,textAlign:e.text_align||'left',justifyContent:(e.kind==='text'||e.kind==='heading'||e.kind==='nav-title'||isNavigation(e))?textJustification(e):''});
    if(e.kind==='image')node.innerHTML=`<span class="image-symbol">▧</span><span>${esc(e.text||'Image placeholder')}</span>`;
    else if(e.kind==='input')node.innerHTML=`<span class="placeholder">${esc(e.placeholder||e.text||'Enter text…')}</span>`;
    else if(isNavigation(e))node.innerHTML=navItems(e).map((label,index)=>`<span class="nav-tab-item${index===0?' active':''}">${esc(label)}</span>`).join('');
    else if(e.kind==='nav-title')node.textContent=e.text||'Title';
    else node.textContent=e.text||(e.kind==='button'?'Button':e.kind==='card'||e.kind==='web-card'?'Card content':e.kind==='heading'?'Heading':'Text');
    renderLayers();
  }
  function makeElement(kind) {
    const presets={text:['Text',220,42,'#ffffff'],heading:['A thoughtful headline',300,56,'#ffffff'],button:['Get started',150,48,'#635bff'],image:['',240,150,'#edf1f7'],input:['',260,48,'#ffffff'],card:['Card content goes here',280,170,'#ffffff'],'web-card':['Feature card',320,190,'#ffffff'],nav:['Home　 Search　 Profile',340,54,'#ffffff'],'web-nav':['Logo　　 Products　 About　 Contact',900,70,'#ffffff'],'nav-title':['Title',state.width,44,'#ffffff']};
    const [text,width,height,fill]=presets[kind]||presets.text;const title=kind==='nav-title';const e={id:crypto.randomUUID(),kind,x:title?0:Math.max(12,(state.width-width)/2),y:title?48:Math.max(20,50+state.elements.length*18),width:title?state.width:width,height,text,fill,color:kind==='button'?'#ffffff':'#172033',font_size:kind==='heading'?28:title?17:16,placeholder:'Email address',radius:kind==='button'?10:8,text_align:title?'center':isNavigation({kind})?'space-between':'left',items:isNavigation({kind})?text.split(/\u3000+|\s{2,}/).map(x=>x.trim()).filter(Boolean):[]};
    saveHistory();state.elements.push(e);state.selected=e.id;render();scheduleSave();
  }
  function removeElement(id){saveHistory();state.elements=state.elements.filter(e=>e.id!==id);state.selected=null;render();scheduleSave();}
  function startDrag(ev){if(ev.target.classList.contains('resize-handle'))return;ev.preventDefault();const node=ev.currentTarget,e=state.elements.find(x=>x.id===node.dataset.id);state.selected=e.id;saveHistory();const scale=parseFloat(getComputedStyle(canvas).transform.match(/matrix\(([^,]+)/)?.[1]||'1');state.drag={type:'move',x:ev.clientX,y:ev.clientY,ex:e.x,ey:e.y,scale,id:e.id};node.setPointerCapture(ev.pointerId);}
  function startResize(ev){ev.preventDefault();ev.stopPropagation();const node=ev.currentTarget.parentElement,e=state.elements.find(x=>x.id===node.dataset.id);saveHistory();const scale=parseFloat(getComputedStyle(canvas).transform.match(/matrix\(([^,]+)/)?.[1]||'1');state.drag={type:'resize',x:ev.clientX,y:ev.clientY,ew:e.width,eh:e.height,scale,id:e.id};node.setPointerCapture(ev.pointerId);}
  document.addEventListener('pointermove',ev=>{if(!state.drag)return;const d=state.drag,e=state.elements.find(x=>x.id===d.id);if(!e)return;const dx=(ev.clientX-d.x)/d.scale,dy=(ev.clientY-d.y)/d.scale;if(d.type==='move'){e.x=Math.max(0,Math.round(d.ex+dx));e.y=Math.max(0,Math.round(d.ey+dy));}else{e.width=Math.max(40,Math.round(d.ew+dx));e.height=Math.max(30,Math.round(d.eh+dy));}render();});
  document.addEventListener('pointerup',()=>{if(state.drag){state.drag=null;scheduleSave();}});
  function scheduleSave(){const screenId=state.screen,elements=clone(state.elements);status('Saving…');clearTimeout(state.saveTimer);state.saveTimer=setTimeout(async()=>{try{const r=await fetch(`/api/screens/${screenId}`,{method:'PUT',headers:{'Content-Type':'application/json'},body:JSON.stringify({version:1,elements})});if(!r.ok)throw Error();if(state.screen===screenId)status('All changes saved');}catch{if(state.screen===screenId)status('Save failed · retrying');}},350);}
  function historyStep(undo){const from=undo?state.history:state.future,to=undo?state.future:state.history;if(!from.length)return;to.push(clone(state.elements));state.elements=from.pop();state.selected=null;render();updateHistoryButtons();scheduleSave();}

  document.querySelectorAll('[data-add]').forEach(b=>b.onclick=()=>makeElement(b.dataset.add));
  $('#undo').onclick=()=>historyStep(true);$('#redo').onclick=()=>historyStep(false);
  canvas.addEventListener('click',ev=>{if(ev.target===canvas){state.selected=null;render();}});
  document.addEventListener('keydown',ev=>{const editing=['INPUT','TEXTAREA','SELECT'].includes(document.activeElement.tagName);if((ev.metaKey||ev.ctrlKey)&&ev.key.toLowerCase()==='z'){ev.preventDefault();historyStep(ev.shiftKey?false:true);}else if(!editing&&(ev.key==='Backspace'||ev.key==='Delete')&&state.selected){removeElement(state.selected);}});
  tabs.forEach(tab=>tab.onclick=()=>loadScreen(tab.dataset.screen,tab));
  $('#preset').onchange=async ev=>{if(!ev.target.value)return;let val=ev.target.value;if(val==='custom'){const w=Number(prompt('Viewport width in pixels',state.width));const h=Number(prompt('Viewport height in pixels',state.height));if(!w||!h)return;val=`${w},${h},${state.frame}`;}const [w,h,frame]=val.split(',');$('#viewport-form [name=width]').value=w;$('#viewport-form [name=height]').value=h;$('#viewport-form [name=frame]').value=frame;$('#viewport-form').requestSubmit();ev.target.value='';};
  $('#viewport-form').onsubmit=async ev=>{ev.preventDefault();const form=new FormData(ev.currentTarget);const r=await fetch(`/screens/${state.screen}/viewport`,{method:'POST',headers:{'Content-Type':'application/x-www-form-urlencoded'},body:new URLSearchParams(form)});if(r.ok){state.width=Number(form.get('width'));state.height=Number(form.get('height'));state.frame=form.get('frame');render();const t=tabs.find(x=>x.dataset.screen===state.screen);if(t){t.dataset.width=state.width;t.dataset.height=state.height;t.dataset.frame=state.frame;} }};
  $('#rename-screen').onsubmit=async ev=>{ev.preventDefault();const name=new FormData(ev.currentTarget).get('name');const r=await fetch(`/screens/${state.screen}/rename`,{method:'POST',headers:{'Content-Type':'application/x-www-form-urlencoded'},body:new URLSearchParams({name})});if(r.ok){$('#active-name').textContent=name;const t=tabs.find(x=>x.dataset.screen===state.screen);if(t){t.textContent=name;t.dataset.name=name;}}};
  $('#delete-screen').onclick=async()=>{if(tabs.length<2){alert('A project needs at least one screen.');return;}if(!confirm('Delete this screen and all its elements?'))return;await fetch(`/screens/${state.screen}/delete`,{method:'POST'});location.reload();};
  document.querySelectorAll('.screen-tab').forEach(t=>{t.draggable=true;t.ondragstart=ev=>{ev.dataTransfer.setData('text/plain',t.dataset.screen);};t.ondragover=ev=>ev.preventDefault();t.ondrop=async ev=>{ev.preventDefault();const from=ev.dataTransfer.getData('text/plain'),to=t.dataset.screen;if(from===to)return;const list=[...document.querySelectorAll('.screen-tab')].map(x=>x.dataset.screen),a=list.indexOf(from),b=list.indexOf(to);list.splice(a,1);list.splice(b,0,from);await fetch(`/screens/${state.project}/order`,{method:'POST',headers:{'Content-Type':'application/json'},body:JSON.stringify({screen_ids:list})});location.reload();};});
  window.addEventListener('resize',fitCanvas);loadScreen(state.screen,tabs[0]);
})();
