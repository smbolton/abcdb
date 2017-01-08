# ABCdb main/views.py
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

import re

from django.http import HttpResponseRedirect
from django.shortcuts import render
from django.utils.html import format_html
from django.views import generic

from main.forms import TitleSearchForm, UploadForm

from main.models import Collection, Instance, Song, Title
from main.abcparser import Tune, ABCParser
import hashlib, operator


# ========== Utility Functions ==========

def _generate_instance_name(instance):
    """Return a string describing the instance, e.g. 'Instance 3 from new.abc of Song 2 (ab37d30)'"""
    iname = 'Instance {} '.format(instance.id)
    collection = Collection.objects.filter(instance=instance.id)
    if collection:
        iname += ' from ' + str(collection[0])[:30]
    iname += ' of ' + str(instance.song)
    return iname


# ========== User-oriented Model Views ==========

class CollectionView(generic.DetailView):
    model = Collection
    template_name = 'main/collection.html'


class InstanceView(generic.DetailView):
    model = Instance
    template_name = 'main/instance.html'

    def collections(self):
        """Collections in which this instance was found."""
        return Collection.objects.filter(instance__id=self.object.pk)

    def other_instances(self):
        """Other instances of this instance's song, as a list of dicts, available in the template
        as view.other_instances."""
        instances = Instance.objects.filter(song__exact=self.object.song_id)
        context = [{ 'pk': i.pk, 'instance': _generate_instance_name(i) }
                       for i in instances if i.pk != self.object.pk]
        return context

    def titles(self):
        """Titles given to this instance's song (all of which may not be present in this
        instance.)"""
        return Title.objects.filter(songs=self.object.song)


class TitleView(generic.DetailView):
    model = Title
    template_name = 'main/title.html'

    def song_instances(self):
        """Songs given this title, as a list of dicts, available in the template as
        view.song_instances."""
        songs = Song.objects.filter(title__id__exact=self.object.pk)
        instances = Instance.objects.filter(song__in=songs)
        context = [{ 'pk': i.pk, 'instance': _generate_instance_name(i) } for i in instances]
        return context


# ========== Title Search ==========

def title_search(request):
    form_class = TitleSearchForm

    if request.method == 'POST':
        form = form_class(request.POST)
        if form.is_valid():
            title = request.POST.get('title')
            # -FIX- what should be done to sanitize title before using it in a query?
            query_set = Title.objects.filter(title__icontains=title).order_by('title')
            return render(request, 'main/title_search-post.html',
                          { 'results': query_set, 'key': title, 'count': len(query_set) })
        else:
            message = ('<div data-alert class="alert-box warning radius">There was a problem '
                       'getting the search string.</div>')
            return render(request, 'main/title_search-post.html', { 'error': message, 'form': form })

    return render(request, 'main/title_search.html', { 'form': form_class, })


# ========== ABC File Upload View and Parser Subclass ==========

class UploadParser(ABCParser):
    def __init__(self, collection=None):
        super().__init__()
        self.collection = collection
        self.status = ''


    def process_tune(self, tune):
        # create the SHA1 digest of the canonical tune, and save it in a Song
        song_digest = hashlib.sha1()
        song_digest.update('\n'.join(map(operator.itemgetter('line'),
                                         tune.canonical)).encode('utf-8'))
        song_digest = song_digest.hexdigest()
        song_inst, new = Song.objects.get_or_create(digest=song_digest)
        song_inst.save()
        if new:
            self.status += format_html("Adding new song {}<br>\n", song_digest[:7])
        else:
            self.status += format_html("Found existing song {}<br>\n", song_digest[:7])
        # save Titles
        for t in tune.T:
            title_inst, new = Title.objects.get_or_create(title=t)
            if new:
                self.status += format_html("Adding new title '{}'<br>\n", t)
            else:
                self.status += format_html("Found existing title '{}'<br>\n", t)
            title_inst.songs.add(song_inst)
        # digest the full tune, and save the tune in an Instance
        full_tune = '\n'.join(tune.full_tune)
        tune_digest = hashlib.sha1()
        tune_digest.update(full_tune.encode('utf-8'))
        tune_digest = tune_digest.hexdigest()
        instance_inst, new = Instance.objects.get_or_create(song=song_inst, digest=tune_digest,
                                                            text=full_tune)
        if new:
            self.status += format_html("Adding new instance {}<br>\n", tune_digest[:7])
        else:
            self.status += format_html("Found existing instance {}<br>\n", tune_digest[:7])
        # save Collection information
        collection_inst, new = Collection.objects.get_or_create(URL=self.collection)
        collection_inst.save()
        collection_inst.instance.add(instance_inst)
        if new:
            self.status += format_html("Adding new collection '{}'<br>\n", self.collection)
        else:
            self.status += format_html("Found existing collection '{}'<br>\n", self.collection)


    def log(self, severity, message, text):
        if isinstance(text, bytes):
            text = text.decode('utf-8', errors='backslashreplace')
        if severity == 'warn':
            self.status += format_html("Warning, line {}: {}: {}<br>\n", str(self.line_number),
                                       message, text)
        elif severity == 'info':
            if 'New tune' in message:
                x = re.sub('\D', '', message) # get tune number
                self.status += format_html("Found start of new tune #{} at line {}<br>\n",
                                        x, str(self.line_number))
        else:  # severity == 'ignore'
            print(severity + ' | ' + str(self.line_number) + ' | ' + message + ' | ' + text)


    def status_append(self, text):
        self.status += text


    def get_status(self):
        return self.status


def upload(request):
    form_class = UploadForm

    if request.method == 'POST':
        form = form_class(request.POST, request.FILES)
        if form.is_valid():
            file = request.FILES['file']
            status = format_html("Processing uploaded file '{}', size {} bytes<br>\n", file.name,
                                 file.size)
            p = UploadParser(collection=file.name)
            p.status_append(status)
            p.parse(file.file)
            return render(request, 'main/upload-post.html', { 'status': p.get_status() })
        else:
            # Django 1.10 does not validate file uploads. Handle this anyway.
            # form.errors is a dict containing error mesages, keys are field names, values are
            # lists of error message strings.
            status = ('<div data-alert class="alert-box warning radius">The file upload was '
                      'invalid. Contact the site administrator if this problem persists.</div>')
            return render(request, 'main/upload-post.html', { 'status': status })

    return render(request, 'main/upload.html', { 'form': form_class, })


# ========== Temporary Views for Development ==========

class CollectionsView(generic.ListView):
    template_name = 'main/temp_collections.html'
    context_object_name = 'collection_list'

    def get_queryset(self):
        return Collection.objects.all().order_by('URL')


class InstancesView(generic.ListView):
    template_name = 'main/temp_instances.html'
    context_object_name = 'instance_list'

    def get_queryset(self):
        return Instance.objects.all().order_by('digest')


class SongsView(generic.ListView):
    template_name = 'main/temp_songs.html'
    context_object_name = 'song_list'

    def get_queryset(self):
        return Song.objects.all().order_by('digest')


class TitlesView(generic.ListView):
    template_name = 'main/temp_titles.html'
    context_object_name = 'title_list'

    def get_queryset(self):
        return Title.objects.all().order_by('title')
