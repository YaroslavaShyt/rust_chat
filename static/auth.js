document.getElementById('login-form').addEventListener('submit', async function (event) {
    event.preventDefault();

    const username = document.getElementById('username').value;
    const password = document.getElementById('password').value;

    const response = await fetch('/login', {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json',
        },
        body: JSON.stringify({ username, password })
    });

    const messageElement = document.getElementById('login-message');
    if (response.ok) {
        const token = await response.json();
        messageElement.textContent = 'Login successful!';
        messageElement.className = 'message success';
        localStorage.setItem('jwt_token', token);
        window.location.href = '/chat';
    } else {
        const errorMessage = await response.text();
        messageElement.textContent = errorMessage;
        messageElement.className = 'message error';
    }

    messageElement.style.display = 'block';
});

document.getElementById('register-form').addEventListener('submit', async function (event) {
    event.preventDefault();

    const username = document.getElementById('reg-username').value;
    const password = document.getElementById('reg-password').value;
    const confirmPassword = document.getElementById('reg-confirm-password').value;

    if (password !== confirmPassword) {
        alert('Passwords do not match');
        return;
    }

    const response = await fetch('/register', {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json',
        },
        body: JSON.stringify({ username, password })
    });

    const messageElement = document.getElementById('register-message');
    if (response.ok) {
        messageElement.textContent = 'Registration successful! You can now log in.';
        messageElement.className = 'message success';
        const token = await response.json();
        localStorage.setItem('jwt_token', token);
        window.location.href = '/chat';
    } else {
        const errorMessage = await response.text();
        messageElement.textContent = errorMessage;
        messageElement.className = 'message error';
        messageElement.style.display = 'block';
    }

});