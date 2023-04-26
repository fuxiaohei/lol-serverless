import React, { useState } from "react";
import { Alert, Form, Button, Container } from "react-bootstrap";
import { Link } from "react-router-dom";
import LoginNavbar from "./LoginNavbar";
import { BsFillCaretLeftSquareFill } from "react-icons/bs";

function LoginEmailPage() {
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [validated, setValidated] = useState(false);
  const [showAlert, setShowAlert] = useState(false);
  const [alertMessage, setAlertMessage] = useState("");

  const handleSubmit = (event) => {
    const form = event.currentTarget;
    const validated = form.checkValidity();
    if (validated === false) {
      event.preventDefault();
      event.stopPropagation();
    }
    setValidated(true);
    setShowAlert(false);
    if (validated) {
      event.preventDefault();
      event.stopPropagation();
      console.log(email, password, validated);
    }
  };

  return (
    <div>
      <LoginNavbar />
      <Container className="login-container">
        <h3 className="login-container-header">Login to Moni-Web</h3>
        <hr />
        <Form
          id="login-email-form"
          noValidate
          validated={validated}
          onSubmit={handleSubmit}
        >
          <Form.Group className="mb-3" controlId="formBasicEmail">
            <Form.Label>Email address</Form.Label>
            <Form.Control
              type="email"
              placeholder="Enter email"
              required
              value={email}
              onChange={(e) => setEmail(e.target.value)}
            />
            <Form.Control.Feedback type="invalid">
              Please provide a valid email.
            </Form.Control.Feedback>
          </Form.Group>

          <Form.Group className="mb-4" controlId="formBasicPassword">
            <Form.Label>Password</Form.Label>
            <Form.Control
              type="password"
              placeholder="Password"
              required
              value={password}
              onChange={(e) => setPassword(e.target.value)}
            />
            <Form.Control.Feedback type="invalid">
              Please input a password.
            </Form.Control.Feedback>
          </Form.Group>
          <Alert variant={"danger"} show={showAlert}>
            {alertMessage}
          </Alert>
          <div className="d-flex mb-4 justify-content-between">
            <Button variant="primary" type="submit" className="w-100">
              Sign up / Sign in
            </Button>
          </div>
          <div className="login-email-link">
            <Link to="/forgotpassword">Forget Password?</Link>
          </div>
        </Form>
        <hr />
        <div className="d-grid gap-2">
          <Button variant="link" href="/login" className="login-email-back">
            <BsFillCaretLeftSquareFill size={16} />
            Other Login Options
          </Button>
        </div>
      </Container>
    </div>
  );
}

export default LoginEmailPage;