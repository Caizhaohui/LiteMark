# Malicious Input Test

<script>alert('xss')</script>

<img src="x" onerror="alert('xss')">

[evil](javascript:alert('xss'))

<iframe src="https://evil.example.com"></iframe>
