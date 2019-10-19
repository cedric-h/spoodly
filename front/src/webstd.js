function display(output) {
	document.getElementById("output").textContent += output + "\n";
}

function input(prompt_string) {
	console.log('before');
	let user_input = prompt(prompt_string);
	console.log(user_input);
	return user_input;
}
