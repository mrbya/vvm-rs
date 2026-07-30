# GitLab CI

Typical GitLab usage looks like this:

```yaml
functional-coverage:
  stage: test
  script:
    - cargo vvm coverage --output target/vvm-functional-coverage --name counter -- test -p vvm-example-counter counter_
  coverage: '/^VVM functional coverage: \d+\.\d{2}%$/'
  artifacts:
    when: always
    paths:
      - target/vvm-functional-coverage/
```

The text report ends with the metric line used by the repository's GitLab regex.
