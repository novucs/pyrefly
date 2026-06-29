/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

use crate::django_testcase;

// A DRF `ModelSerializer` subclass defines its own nested `Meta` options class
// without inheriting from `ModelSerializer.Meta`. This is the framework's
// configuration convention, not a Liskov-substitutable member override, so it
// must not be flagged as an inconsistent override.
django_testcase!(
    test_model_serializer_meta_override_allowed,
    r#"
from django.db import models
from rest_framework import serializers

class MyModel(models.Model):
    name = models.CharField(max_length=10)

class MySerializer(serializers.ModelSerializer):
    class Meta:
        model = MyModel
        fields = ["name"]
"#,
);
