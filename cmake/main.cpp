#include <format>
#include <print>
#include <gtk/gtk.h>

static void my_activate(GtkApplication* app, gpointer data)
{
    GtkWidget* window = gtk_application_window_new(app);
    gtk_window_set_title(GTK_WINDOW(window), "GTK4 Demo");
    gtk_window_set_default_size(GTK_WINDOW(window), 400, 300);
    gtk_widget_set_visible(window, true);
}

int main(int argc, char** argv)
{
    std::println("Initializing GTK...");
    GtkApplication* app = gtk_application_new("org.example.app", G_APPLICATION_FLAGS_NONE);
    g_signal_connect(app, "activate", G_CALLBACK(my_activate), NULL);
    int status = g_application_run(G_APPLICATION(app), argc, argv);
    g_object_unref(app);
    return status;
}
