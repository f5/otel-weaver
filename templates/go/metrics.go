// Univariate metrics
{% for metric in schema.resource_metrics.univariate_metrics %}
  {{metric.name}}
{% endfor %}

// Multivariate metrics
{% for mmetric in schema.resource_metrics.multivariate_metrics %}
  {{mmetric.id}}
{% endfor %}