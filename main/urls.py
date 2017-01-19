# ABCdb main/urls.py
#
# Copyright © 2017 Sean Bolton.
#
# Permission is hereby granted, free of charge, to any person obtaining
# a copy of this software and associated documentation files (the
# "Software"), to deal in the Software without restriction, including
# without limitation the rights to use, copy, modify, merge, publish,
# distribute, sublicense, and/or sell copies of the Software, and to
# permit persons to whom the Software is furnished to do so, subject to
# the following conditions:
#
# The above copyright notice and this permission notice shall be
# included in all copies or substantial portions of the Software.
#
# THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,
# EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF
# MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND
# NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE
# LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION
# OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION
# WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.

from django.conf.urls import url

from . import views


urlpatterns = [
    url(r'^collection/(?P<pk>[0-9]{1,9})/$', views.CollectionView.as_view()),
    url(r'^collections/$', views.CollectionsView.as_view()),
    url(r'^download/(?P<pk>[0-9]{1,9})/$', views.download),
    url(r'^instance/(?P<pk>[0-9]{1,9})/$', views.InstanceView.as_view()),
    url(r'^search/$', views.title_search, name='title_search'),
    url(r'^song/(?P<pk>[0-9]{1,9})/$', views.SongView.as_view()),
    url(r'^title/(?P<pk>[0-9]{1,9})/$', views.TitleView.as_view()),
    url(r'^titles/$', views.TitlesView.as_view()),
    url(r'^upload/$', views.upload, name='upload'),
    # temporary
    url(r'^temp_songs/$', views.SongsView.as_view()),
    url(r'^temp_instances/$', views.InstancesView.as_view()),
]
