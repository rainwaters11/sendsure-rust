
const result = document.getElementById('result');
const cont = document.getElementById('continue');
const form = document.getElementById('intent-form');
const evaluateButton = document.getElementById('evaluate');
const resetButton = document.getElementById('reset');
const scenarioBox = document.getElementById('scenarios');
const checkButton = document.getElementById('check');
const swapSlippageRange = document.getElementById('swap-slippage-range');
const actionButtons = [...document.querySelectorAll('.actions button')];

const evaluateDefaultLabel = (evaluateButton.textContent || 'Evaluate intent').trim();
const resultDefault = 'Choose a demo scenario or enter transaction details to begin.';
const actionChangedText = 'Transaction details changed. Run the preflight check again.';
const evaluatingText = 'Evaluating with deterministic Rust rules...';

const fieldNames = [
    'action_type',
    'source_network',
    'destination_network',
    'asset_symbol',
    'asset_identifier',
    'destination_address',
    'expected_destination_address',
    'entered_destination_tag_or_memo',
    'expected_destination_tag_or_memo',
    'contract_address',
    'approval_amount_or_scope',
    'swap_slippage_percent',
    'transaction_origin',
];

const allIntentFields = [...fieldNames, 'asset_was_unsolicited'];
const fieldVisibility = {
    SEND: [
        'source_network',
        'destination_network',
        'asset_symbol',
        'asset_identifier',
        'destination_address',
        'expected_destination_address',
        'entered_destination_tag_or_memo',
        'expected_destination_tag_or_memo',
        'transaction_origin',
    ],
    SWAP: [
        'source_network',
        'destination_network',
        'asset_symbol',
        'asset_identifier',
        'destination_address',
        'expected_destination_address',
        'swap_slippage_percent',
        'transaction_origin',
    ],
    APPROVE: [
        'source_network',
        'asset_symbol',
        'asset_identifier',
        'contract_address',
        'approval_amount_or_scope',
        'transaction_origin',
    ],
    SIGN: ['source_network', 'contract_address', 'transaction_origin', 'asset_was_unsolicited'],
};

const formFields = Object.fromEntries([...form.elements].filter((el) => el.name).map((el) => [el.name, el]));

let selectedScenarioIndex = null;
let scenarioButtons = [];
let selectedAction = 'SEND';
let activeRequestController = null;
let requestGeneration = 0;
let isProgrammaticUpdate = false;

function blank(value) {
    return value == null ? '' : String(value);
}

function normalizeAction(action) {
    const next = blank(action).toUpperCase();
    return fieldVisibility[next] ? next : 'SEND';
}

function nullableText(value) {
    if (value == null) {
        return null;
    }
    const trimmed = String(value).trim();
    return trimmed === '' ? null : trimmed;
}

function nullableNumber(value) {
    const text = nullableText(value);
    if (text == null) {
        return null;
    }
    const parsed = Number(text);
    return Number.isFinite(parsed) ? parsed : null;
}

function withProgrammaticUpdate(fn) {
    const previous = isProgrammaticUpdate;
    isProgrammaticUpdate = true;
    try {
        return fn();
    } finally {
        isProgrammaticUpdate = previous;
    }
}

function fieldContainer(field) {
    if (!field) {
        return null;
    }
    return field.closest('[data-field-group]') || (field.type === 'checkbox' && field.parentElement ? field.parentElement : field);
}

function clearFieldValue(name) {
    const field = formFields[name];
    if (!field) {
        return;
    }
    if (field.type === 'checkbox') {
        field.checked = false;
        return;
    }
    if (name === 'swap_slippage_percent') {
        setSwapSlippageValue(null);
        return;
    }
    if (name !== 'action_type') {
        field.value = '';
    }
}

function setFieldVisibility(name, visible) {
    const field = formFields[name];
    if (!field) {
        return;
    }
    const container = fieldContainer(field);
    if (container) {
        container.hidden = !visible;
        container.classList.toggle('is-hidden', !visible);
    }
}

function setSwapSlippageValue(value) {
    const text = value == null ? '' : String(value).trim();
    const hasExplicitValue = text !== '';
    const nextFieldValue = hasExplicitValue ? text : '';
    const nextRangeValue = hasExplicitValue ? text : swapSlippageRange?.defaultValue || '0';
    if (formFields.swap_slippage_percent) {
        formFields.swap_slippage_percent.value = nextFieldValue;
    }
    if (swapSlippageRange) {
        swapSlippageRange.value = nextRangeValue;
    }
}

function isFieldVisible(action, name) {
    return (fieldVisibility[action] || fieldVisibility.SEND).includes(name);
}

function focusFirstVisibleField(action) {
    const order = (fieldVisibility[action] || fieldVisibility.SEND).slice();
    const firstName = order.find((name) => {
        if (name === 'action_type') {
            return false;
        }
        const field = formFields[name];
        if (!field) {
            return false;
        }
        const container = fieldContainer(field);
        return !(container && container.hidden);
    });
    const firstField = firstName ? formFields[firstName] : formFields.source_network;
    if (firstField && typeof firstField.focus === 'function') {
        firstField.focus();
    }
}

function updateFieldVisibility(action) {
    allIntentFields.forEach((name) => {
        if (name === 'action_type') {
            return;
        }
        setFieldVisibility(name, isFieldVisible(action, name));
    });
}

function clearIrrelevantFieldValues(action) {
    allIntentFields.forEach((name) => {
        if (name === 'action_type') {
            return;
        }
        if (!isFieldVisible(action, name)) {
            clearFieldValue(name);
        }
    });
}

function clearAllIntentFieldValues() {
    allIntentFields.forEach((name) => {
        if (name !== 'action_type') {
            clearFieldValue(name);
        }
    });
}

function setActionState(action, { clearIrrelevant = false, clearAll = false, focus = false } = {}) {
    selectedAction = normalizeAction(action);
    if (clearAll) {
        clearAllIntentFieldValues();
    } else if (clearIrrelevant) {
        clearIrrelevantFieldValues(selectedAction);
    }
    actionButtons.forEach((button) => {
        button.classList.toggle('active', button.dataset.action === selectedAction);
    });
    formFields.action_type.value = selectedAction;
    updateFieldVisibility(selectedAction);
    if (focus) {
        focusFirstVisibleField(selectedAction);
    }
}

function updateScenarioHighlight() {
    scenarioButtons.forEach((button, index) => {
        button.classList.toggle('scenario-selected', index === selectedScenarioIndex);
    });
}

function clearUiMessages() {
    document
        .querySelectorAll('[data-validation-message],.validation-message,.api-error,.error-message')
        .forEach((el) => {
            el.textContent = '';
            el.classList.remove('show');
            el.hidden = true;
        });
    document
        .querySelectorAll(
            '[data-risk-ack],[data-wallet-handoff],[data-completion-message],#risk-acknowledgment,#wallet-handoff,#completion-message'
        )
        .forEach((el) => {
            if ('checked' in el) {
                el.checked = false;
            }
            el.textContent = '';
            el.classList.remove('show');
            el.hidden = true;
        });
}

function closeOpenModals() {
    document.querySelectorAll('dialog[open]').forEach((dialog) => {
        if (typeof dialog.close === 'function') {
            dialog.close();
        } else {
            dialog.removeAttribute('open');
        }
    });
    document.querySelectorAll('.modal.open,.panel.open,[data-open="true"]').forEach((el) => {
        el.classList.remove('open');
        el.setAttribute('data-open', 'false');
        el.hidden = true;
    });
}

function clearPendingEvaluation() {
    requestGeneration += 1;
    if (activeRequestController) {
        activeRequestController.abort();
        activeRequestController = null;
    }
}

function invalidateActionEvaluationState() {
    clearPendingEvaluation();
    selectedScenarioIndex = null;
    updateScenarioHighlight();
    clearUiMessages();
    closeOpenModals();
    result.className = 'card';
    result.textContent = actionChangedText;
    applyContinueState();
    evaluateButton.disabled = false;
    checkButton.disabled = false;
    evaluateButton.textContent = evaluateDefaultLabel;
}

function applyManualActionChange(action) {
    const nextAction = normalizeAction(action);
    if (nextAction === selectedAction) {
        return;
    }
    withProgrammaticUpdate(() => {
        setActionState(nextAction, { clearIrrelevant: true, focus: true });
    });
    invalidateActionEvaluationState();
}

function shouldInvalidateForManualEdit(target) {
    if (isProgrammaticUpdate || !target || !target.name) {
        return false;
    }
    if (target.name === 'action_type') {
        return false;
    }
    return true;
}

function handleManualFieldInput(event) {
    const target = event.target;
    if (!shouldInvalidateForManualEdit(target)) {
        return;
    }
    const type = (target.type || '').toLowerCase();
    if (type === 'checkbox') {
        return;
    }
    invalidateActionEvaluationState();
}

function handleManualFieldChange(event) {
    const target = event.target;
    if (!shouldInvalidateForManualEdit(target)) {
        return;
    }
    const tag = (target.tagName || '').toUpperCase();
    const type = (target.type || '').toLowerCase();
    const isCheckbox = tag === 'INPUT' && type === 'checkbox';
    const isSelect = tag === 'SELECT';
    if (!isCheckbox && !isSelect) {
        return;
    }
    invalidateActionEvaluationState();
}

function populateScenario(intent, scenarioName) {
    fieldNames.forEach((name) => {
        const field = formFields[name];
        if (!field) {
            return;
        }
        if (name === 'swap_slippage_percent') {
            setSwapSlippageValue(intent?.[name]);
            return;
        }
        if (name === 'expected_destination_tag_or_memo') {
            field.value = scenarioName === 'XRP destination tag mismatch' ? '482901' : blank(intent?.[name]);
            return;
        }
        field.value = blank(intent?.[name]);
    });
    formFields.asset_was_unsolicited.checked = Boolean(intent?.asset_was_unsolicited);
}

function valueForField(name) {
    if (!isFieldVisible(selectedAction, name)) {
        return null;
    }
    if (name === 'swap_slippage_percent') {
        return nullableNumber(formFields.swap_slippage_percent.value);
    }
    return nullableText(formFields[name].value);
}

function buildIntentFromForm() {
    return {
        action_type: selectedAction,
        source_network: valueForField('source_network'),
        destination_network: valueForField('destination_network'),
        asset_symbol: valueForField('asset_symbol'),
        asset_identifier: valueForField('asset_identifier'),
        destination_address: valueForField('destination_address'),
        expected_destination_address: valueForField('expected_destination_address'),
        entered_destination_tag_or_memo: valueForField('entered_destination_tag_or_memo'),
        expected_destination_tag_or_memo: valueForField('expected_destination_tag_or_memo'),
        contract_address: valueForField('contract_address'),
        approval_amount_or_scope: valueForField('approval_amount_or_scope'),
        swap_slippage_percent: valueForField('swap_slippage_percent'),
        transaction_origin: valueForField('transaction_origin'),
        asset_was_unsolicited: isFieldVisible(selectedAction, 'asset_was_unsolicited')
            ? Boolean(formFields.asset_was_unsolicited.checked)
            : false,
    };
}

function applyContinueState(decision) {
    if (decision === 'STOP') {
        cont.disabled = true;
        cont.textContent = 'Fix issue first';
        return;
    }
    if (decision === 'REVIEW') {
        cont.disabled = false;
        cont.textContent = 'I understand the risk';
        return;
    }
    if (decision === 'READY') {
        cont.disabled = false;
        cont.textContent = 'Ready for wallet review';
        return;
    }
    cont.disabled = true;
    cont.textContent = 'Continue';
}

function escapeHtml(value) {
    return blank(value)
        .replace(/&/g, '&amp;')
        .replace(/</g, '&lt;')
        .replace(/>/g, '&gt;')
        .replace(/"/g, '&quot;')
        .replace(/'/g, '&#39;');
}

function decisionSummary(decision) {
    if (decision === 'READY') {
        return 'Details match the stated intent. Review your wallet before continuing.';
    }
    if (decision === 'REVIEW') {
        return 'A risk needs your attention before you continue.';
    }
    if (decision === 'STOP') {
        return 'Do not continue until this issue is corrected.';
    }
    return '';
}

function setResultIdle() {
    result.className = 'card';
    result.textContent = resultDefault;
    applyContinueState();
    checkButton.disabled = false;
}

function setResultLoading() {
    result.className = 'card';
    result.textContent = evaluatingText;
    evaluateButton.disabled = true;
    checkButton.disabled = true;
    evaluateButton.textContent = 'Evaluating...';
    applyContinueState();
}

function renderResult(payload) {
    result.className = 'card ' + payload.decision;
    const decision = escapeHtml(payload.decision);
    const summary = escapeHtml(decisionSummary(payload.decision));
    const ruleId = escapeHtml(payload.triggered_rule_id);
    const explanation = escapeHtml(payload.explanation);
    const nextStep = escapeHtml(payload.recommended_next_step);
    result.innerHTML = `<div class="decision-banner"><div class="decision-header"><h2 class="decision-title">${decision}</h2><span class="decision-badge ${decision}">${decision}</span></div><p class="decision-summary">${summary}</p><p class="rule-line"><strong>Rule:</strong> <span class="rule-pill">${ruleId}</span></p><div class="decision-body"><p>${explanation}</p><p><b>Recommended next step:</b> ${nextStep}</p></div><div class="trust-row" aria-label="Trust indicators"><span class="trust-chip">Deterministic Rust rules</span><span class="trust-chip">No custody</span><span class="trust-chip">No transaction sent</span></div></div>`;
    applyContinueState(payload.decision);
    evaluateButton.disabled = false;
    checkButton.disabled = false;
    evaluateButton.textContent = evaluateDefaultLabel;
    result.scrollIntoView({ behavior: 'smooth', block: 'start' });
}

function renderError(message) {
    result.className = 'card';
    result.replaceChildren();
    const heading = document.createElement('h2');
    heading.textContent = 'Error';
    const paragraph = document.createElement('p');
    paragraph.textContent = message;
    result.append(heading, paragraph);
    applyContinueState();
    evaluateButton.disabled = false;
    checkButton.disabled = false;
    evaluateButton.textContent = evaluateDefaultLabel;
    result.scrollIntoView({ behavior: 'smooth', block: 'start' });
}

async function evaluateIntent(intent) {
    if (activeRequestController) {
        activeRequestController.abort();
        activeRequestController = null;
    }
    const myGeneration = ++requestGeneration;
    const controller = new AbortController();
    activeRequestController = controller;
    setResultLoading();
    try {
        const response = await fetch('/api/evaluate', {
            method: 'POST',
            cache: 'no-store',
            signal: controller.signal,
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(intent),
        });
        if (myGeneration !== requestGeneration) {
            return;
        }
        if (!response.ok) {
            throw new Error(`Request failed with status ${response.status}`);
        }
        const payload = await response.json();
        if (myGeneration !== requestGeneration) {
            return;
        }
        renderResult(payload);
    } catch (error) {
        if (error && error.name === 'AbortError') {
            return;
        }
        if (myGeneration !== requestGeneration) {
            return;
        }
        renderError(`Unable to evaluate intent right now. Please try again in a moment. (${error.message})`);
    } finally {
        if (activeRequestController === controller) {
            activeRequestController = null;
        }
    }
}

async function evaluateFromForm() {
    await evaluateIntent(buildIntentFromForm());
}

function resetExperience() {
    clearPendingEvaluation();
    withProgrammaticUpdate(() => {
        HTMLFormElement.prototype.reset.call(form);
        clearAllIntentFieldValues();
        setActionState('SEND', { clearIrrelevant: false, focus: false });
    });
    selectedScenarioIndex = null;
    updateScenarioHighlight();
    clearUiMessages();
    closeOpenModals();
    setResultIdle();
    evaluateButton.disabled = false;
    checkButton.disabled = false;
    evaluateButton.textContent = evaluateDefaultLabel;
    form.scrollIntoView({ behavior: 'smooth', block: 'start' });
    focusFirstVisibleField('SEND');
}

actionButtons.forEach((button) => {
    button.addEventListener('click', () => {
        applyManualActionChange(button.dataset.action || 'SEND');
    });
});

formFields.action_type.addEventListener('change', () => {
    applyManualActionChange(formFields.action_type.value);
});

form.addEventListener('input', handleManualFieldInput);
form.addEventListener('change', handleManualFieldChange);

if (swapSlippageRange && formFields.swap_slippage_percent) {
    swapSlippageRange.addEventListener('input', () => {
        withProgrammaticUpdate(() => {
            formFields.swap_slippage_percent.value = swapSlippageRange.value;
        });
        invalidateActionEvaluationState();
    });
    formFields.swap_slippage_percent.addEventListener('input', () => {
        withProgrammaticUpdate(() => {
            swapSlippageRange.value = formFields.swap_slippage_percent.value || '0';
        });
    });
}

checkButton.addEventListener('click', async () => {
    form.scrollIntoView({ behavior: 'smooth', block: 'start' });
    await evaluateFromForm();
});
resetButton.addEventListener('click', resetExperience);

form.addEventListener('submit', async (event) => {
    event.preventDefault();
    await evaluateFromForm();
});

withProgrammaticUpdate(() => {
    setActionState('SEND');
});
setResultIdle();

fetch('/api/scenarios', { cache: 'no-store' })
    .then((response) => {
        if (!response.ok) {
            throw new Error(`Request failed with status ${response.status}`);
        }
        return response.json();
    })
    .then((scenarios) => {
        scenarioBox.innerHTML = '';
        scenarioButtons = [];
        scenarios.forEach((scenario, index) => {
            const button = document.createElement('button');
            button.type = 'button';
            button.textContent = `${index + 1}. ${scenario.name}`;
            button.addEventListener('click', async () => {
                selectedScenarioIndex = index;
                updateScenarioHighlight();
                withProgrammaticUpdate(() => {
                    setActionState(blank(scenario.intent?.action_type) || 'SEND', {
                        clearIrrelevant: false,
                        focus: false,
                    });
                    populateScenario(scenario.intent, scenario.name);
                });
                await evaluateFromForm();
            });
            scenarioBox.appendChild(button);
            scenarioButtons.push(button);
        });
    })
    .catch(() => {
        renderError('Unable to load demo scenarios. Refresh and try again.');
    });
