import zipfile
import xml.dom.minidom
import sys
import os

def debug_odt(filepath):
    if not os.path.exists(filepath):
        print(f"Error: File not found: {filepath}")
        return

    try:
        with zipfile.ZipFile(filepath, 'r') as z:
            print(f"Files in archive: {z.namelist()}")
            
            if 'META-INF/manifest.xml' in z.namelist():
                print("\n--- META-INF/manifest.xml ---")
                print(z.read('META-INF/manifest.xml').decode('utf-8'))
            else:
                print("\nWarning: No META-INF/manifest.xml found")

            if 'styles.xml' in z.namelist():
                print("\n--- styles.xml (Style Names) ---")
                content = z.read('styles.xml')
                dom = xml.dom.minidom.parseString(content)
                styles = dom.getElementsByTagName('style:style')
                for s in styles:
                    print(f"Style: {s.getAttribute('style:name')} (Family: {s.getAttribute('style:family')})")
                
                defaults = dom.getElementsByTagName('style:default-style')
                for s in defaults:
                    print(f"Default Style: Family={s.getAttribute('style:family')}")

            if 'content.xml' in z.namelist():
                print("\n--- content.xml (Snippet) ---")
                content = z.read('content.xml')
                dom = xml.dom.minidom.parseString(content)
                # Just print the first 500 chars of pretty xml to avoid spam
                pretty = dom.toprettyxml()
                print(pretty[:2000] + "...")
            else:
                print("Error: content.xml not found in archive")
    except zipfile.BadZipFile:
        print("Error: Not a valid zip file")
    except Exception as e:
        print(f"Error: {e}")

if __name__ == "__main__":
    if len(sys.argv) < 2:
        print("Usage: python3 debug_odt.py <path_to_odt_file>")
    else:
        debug_odt(sys.argv[1])
