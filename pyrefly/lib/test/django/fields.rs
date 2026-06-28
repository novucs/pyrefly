/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

use crate::django_testcase;
use crate::test::django::util::django_env;
use crate::test::util::TestEnv;
use crate::testcase;

django_testcase!(
    test_textfield_nullable,
    r#"
from django.db import models

class Group(models.Model):
    name = models.TextField()

class Customer(models.Model):
    name = models.TextField(null=True)

def test():
    c = Customer.objects.create()
    c.name = None
"#,
);

django_testcase!(
    test_get_foo_display,
    r#"
from typing import assert_type

from django.db import models

class Person(models.Model):
    SHIRT_SIZES = {
        "S": "Small",
        "M": "Medium",
        "L": "Large",
    }
    name = models.CharField(max_length=60)
    shirt_size = models.CharField(max_length=2, choices=SHIRT_SIZES)

p = Person(name="Fred Flintstone", shirt_size="L")
p.save()
assert_type(p.shirt_size, str) 
assert_type(p.get_shirt_size_display(), str)
"#,
);

django_testcase!(
    test_charfield_choices_inline_tuple,
    r#"
from typing import assert_type, Literal

from django.db import models

class Card(models.Model):
    suit = models.CharField(
        max_length=100,
        choices=(
            ("CLUBS", "Clubs"),
            ("SPADES", "Spades"),
            ("HEARTS", "Hearts"),
            ("DIAMONDS", "Diamonds"),
        ),
    )

card = Card(suit="CLUBS")
assert_type(card.suit, Literal["CLUBS", "SPADES", "HEARTS", "DIAMONDS"])
"#,
);

// We can consider narrowing the type further
// but it's unclear if it's worth doing so since
// django stubs give type Any for the aggregate values
django_testcase!(
    test_int_field,
    r#"
from django.db import models
from typing import assert_type, Any

class Default(models.Model):
    int_field = models.IntegerField(default=0)

total_sum_typed = Default.objects.aggregate(
        total=models.Sum("int_field", output_field=models.IntegerField())
)
assert_type(total_sum_typed, dict[str, Any]) 
"#,
);

django_testcase!(
    test_queryset_as_manager_preserves_custom_methods,
    r#"
from django.db import models

class NotificationQuerySet(models.QuerySet["Notification"]):
    def resolved(self):
        return self.filter(resolved_at__isnull=False)

class Notification(models.Model):
    resolved_at = models.DateTimeField(null=True, blank=True)
    objects = NotificationQuerySet.as_manager()

Notification.objects.resolved()
Notification.objects.all().resolved()
"#,
);

// Same as above, but the manager is created once at module level and assigned to
// `objects` through a name. This is the common `Manager = QS.as_manager()` pattern.
django_testcase!(
    test_queryset_as_manager_through_alias_preserves_custom_methods,
    r#"
from django.db import models

class NotificationQuerySet(models.QuerySet["Notification"]):
    def resolved(self):
        return self.filter(resolved_at__isnull=False)

NotificationManager = NotificationQuerySet.as_manager()

class Notification(models.Model):
    resolved_at = models.DateTimeField(null=True, blank=True)
    objects = NotificationManager

Notification.objects.resolved()
Notification.objects.all().resolved()
"#,
);

fn django_env_with_manager_module() -> TestEnv {
    let mut env = django_env();
    env.add(
        "managers",
        r#"
from django.db import models

class NotificationQuerySet(models.QuerySet):
    def resolved(self):
        return self.filter(resolved_at__isnull=False)

NotificationManager = NotificationQuerySet.as_manager()
"#,
    );
    env
}

// The manager is defined and created in a *separate* module and imported, which is
// the most common real-world shape. The imported manager must still expose the
// queryset's custom methods on `objects`.
testcase!(
    test_queryset_as_manager_imported_preserves_custom_methods,
    django_env_with_manager_module(),
    r#"
from django.db import models
from managers import NotificationManager

class Notification(models.Model):
    resolved_at = models.DateTimeField(null=True, blank=True)
    objects = NotificationManager

Notification.objects.resolved()
Notification.objects.all().resolved()
"#,
);
