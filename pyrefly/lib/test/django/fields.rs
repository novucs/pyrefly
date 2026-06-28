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

// `QuerySet.as_manager()` is modeled as a *manager* (not the queryset): the queryset's
// own methods are grafted on (returning the queryset), the standard manager API keeps
// the precise model, and — soundly — the manager is NOT iterable/subscriptable.
django_testcase!(
    test_queryset_as_manager_is_sound_manager,
    r#"
from typing import assert_type
from django.db import models

class NotificationQuerySet(models.QuerySet["Notification"]):
    def resolved(self):
        return self.filter(resolved_at__isnull=False)

class Notification(models.Model):
    resolved_at = models.DateTimeField(null=True, blank=True)
    objects = NotificationQuerySet.as_manager()

# Custom queryset method is available on the manager and returns the queryset.
assert_type(Notification.objects.resolved(), NotificationQuerySet)
assert_type(Notification.objects.resolved().resolved(), NotificationQuerySet)
# Standard queryset methods return the custom queryset, so chaining a custom method works.
assert_type(Notification.objects.all(), NotificationQuerySet)
assert_type(Notification.objects.all().resolved(), NotificationQuerySet)
# Methods keep the precise model.
assert_type(Notification.objects.get(), Notification)
# A manager is not a queryset: iteration and subscription are rejected.
for n in Notification.objects:  # E: is not iterable
    pass
Notification.objects[0]  # E: Cannot index into
"#,
);

// The manager is created once at module level and assigned to `objects` through a name —
// the common `Manager = QS.as_manager()` pattern. The synthesized manager type flows
// through the name binding.
django_testcase!(
    test_queryset_as_manager_through_alias,
    r#"
from typing import assert_type
from django.db import models

class NotificationQuerySet(models.QuerySet["Notification"]):
    def resolved(self):
        return self.filter(resolved_at__isnull=False)

NotificationManager = NotificationQuerySet.as_manager()

class Notification(models.Model):
    resolved_at = models.DateTimeField(null=True, blank=True)
    objects = NotificationManager

assert_type(Notification.objects.resolved(), NotificationQuerySet)
assert_type(Notification.objects.get(), Notification)
Notification.objects[0]  # E: Cannot index into
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

// The manager is defined and created in a *separate* module and imported — the most
// common real-world shape. The imported manager still exposes the queryset's methods
// and stays a (non-iterable) manager.
testcase!(
    test_queryset_as_manager_imported,
    django_env_with_manager_module(),
    r#"
from typing import assert_type
from django.db import models
from managers import NotificationManager, NotificationQuerySet

class Notification(models.Model):
    resolved_at = models.DateTimeField(null=True, blank=True)
    objects = NotificationManager

assert_type(Notification.objects.resolved(), NotificationQuerySet)
for n in Notification.objects:  # E: is not iterable
    pass
"#,
);

// `Manager.from_queryset(QS)` yields the manager *class*; instances expose the
// queryset's methods and are not iterable.
django_testcase!(
    test_manager_from_queryset,
    r#"
from typing import assert_type
from django.db import models

class NotificationQuerySet(models.QuerySet["Notification"]):
    def resolved(self):
        return self.filter(resolved_at__isnull=False)

class Notification(models.Model):
    resolved_at = models.DateTimeField(null=True, blank=True)
    objects = models.Manager.from_queryset(NotificationQuerySet)()

assert_type(Notification.objects.resolved(), NotificationQuerySet)
assert_type(Notification.objects.get(), Notification)
Notification.objects[0]  # E: Cannot index into
"#,
);
